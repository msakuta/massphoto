use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
};

use actix_web::web;
use dunce::canonicalize;
use rusqlite::Connection;

use crate::{
    cache::{CacheEntry, CachePayload},
    files::load_cache,
    measure_time, Args, MyData,
};

const CURRENT_VERSION: (usize, usize, usize) = (0, 1, 0);

pub(crate) fn init_db(args: &Args) -> anyhow::Result<web::Data<MyData>> {
    let path = Path::new(&args.path);
    let db_path = path.join("sqliter.db");
    let conn = Connection::open(&db_path)?;

    if !table_exists(&conn, "schema_version") {
        // Keep track of when to apply migration
        conn.execute_batch(&format!(
            "CREATE TABLE schema_version (
            major INTEGER NOT NULL,
            minor INTEGER NOT NULL,
            release INTEGER NOT NULL
        );
        INSERT INTO schema_version (major, minor, release) VALUES ({}, {}, {});",
            CURRENT_VERSION.0, CURRENT_VERSION.1, CURRENT_VERSION.2,
        ))
        .unwrap();
        println!("table \"schema_version\" created!");
    } else {
        let version: (usize, usize, usize) = conn
            .query_row("SELECT * from schema_version", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap();
        if CURRENT_VERSION.0 < version.0 {
            return Err(anyhow::anyhow!(
                "The database schema is newer than this program."
            ));
        }
    }

    if !table_exists(&conn, "file") {
        conn.execute(
            "CREATE table file (
            path TEXT PRIMARY KEY,
            modified REAL,
            desc TEXT,
            data BLOB
        )",
            [],
        )
        .unwrap();
        println!("table \"file\" created!");
    }

    if !table_exists(&conn, "album") {
        conn.execute(
            "CREATE TABLE album (
                path TEXT PRIMARY KEY,
                password TEXT,
                desc TEXT,
                owner INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();
        println!("table \"album\" created!");
    }

    crate::user::init_table(&conn)?;

    println!("tables opened");

    let mut cache = HashMap::new();
    load_cache(&mut cache, &conn, &Path::new(path))?;

    let data = web::Data::new(MyData {
        path: Mutex::new(canonicalize(PathBuf::from(path))?),
        cache: Mutex::new(cache),
        conn: Mutex::new(conn),
        // stats: Mutex::default(),
        sessions: RwLock::default(),
    });
    Ok(data)
}

pub(crate) fn write_db(
    data: &MyData,
    cache: &mut HashMap<PathBuf, CacheEntry>,
) -> Result<(), rusqlite::Error> {
    let filter = |(_, entry): &(&PathBuf, &CacheEntry)| entry.new;
    println!(
        "Saving {}/{} cached thumbnails...",
        cache.iter().filter(filter).count(),
        cache.len()
    );

    let mut db = data.conn.lock().unwrap();

    let tx = db.transaction()?;

    for (key, value) in cache.iter_mut().filter(|(_, entry)| entry.new) {
        let path_str = key.as_os_str();
        match &mut value.payload {
            CachePayload::File(payload) => {
                let byte_contents = std::mem::take(&mut payload.data);
                if tx
                    .query_row(
                        "SELECT path FROM file WHERE path=?1",
                        [path_str.to_str()],
                        |row| row.get::<_, String>(0),
                    )
                    .is_ok()
                {
                    tx.execute(
                        "UPDATE file SET modified = ?2, data = ?3 WHERE path = ?1",
                        rusqlite::params![path_str.to_str(), value.modified, byte_contents],
                    )?;
                } else {
                    tx.execute(
                        "INSERT INTO file (path, modified, data) VALUES (?1, ?2, ?3)",
                        rusqlite::params![path_str.to_str(), value.modified, byte_contents],
                    )?;
                }
            }
            CachePayload::Album(_value) => {
                // TODO: currently, albums don't have thumbnail caches
            }
        }
        value.new = false;
    }
    tx.commit()?;
    Ok(())
}

pub(crate) fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
        [name],
        |row| row.get(0) as rusqlite::Result<String>,
    )
    .is_ok()
}

pub(crate) async fn periodic_cleanup(data: web::Data<MyData>, cleanup_period: u64) {
    let mut interval = actix_rt::time::interval(std::time::Duration::from_secs(cleanup_period));
    let mut i = 0;
    loop {
        interval.tick().await;
        let mut all_files = 0;
        let (mut cached_files, mut cache_size) = (0, 0);
        let (res, tim) = measure_time(|| -> rusqlite::Result<()> {
            let Ok(mut cache) = data.cache.try_lock() else {
                return Ok(());
            };
            all_files = cache.len();
            (cached_files, cache_size) = cache.values().fold((0, 0), |mut acc, entry| {
                match entry.payload {
                    CachePayload::File(ref f) => {
                        if !f.data.is_empty() {
                            acc.0 += 1;
                            acc.1 += f.data.len();
                        }
                    }
                    _ => {}
                }
                acc
            });
            i += 1;
            write_db(&data, &mut cache)?;
            Ok(())
        });
        println!(
            "Periodic Housekeeping {i} in {tim} s: {}/{} images, est. size: {}kb",
            cached_files,
            all_files,
            cache_size as f64 / 1024.
        );
        if let Err(e) = res {
            // A failure to saving the file is not a fatal error. Print on console and carry on.
            println!("Error in periodic write_db: {e}");
        }
    }
}
