use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use actix_web::web;
use dunce::canonicalize;
use rusqlite::Connection;

use crate::{files::load_cache, CacheEntry, CachePayload, MyData};

pub(crate) fn init_db(path: &Path) -> anyhow::Result<web::Data<MyData>> {
    let conn = Connection::open("sqliter.db")?;

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
                desc TEXT
            )",
            [],
        )
        .unwrap();
        println!("table \"album\" created!");
    }

    println!("tables opened");

    let mut cache = HashMap::new();
    load_cache(&mut cache, &conn, &Path::new(path))?;

    let data = web::Data::new(MyData {
        path: Mutex::new(canonicalize(PathBuf::from(path))?),
        cache: Mutex::new(cache),
        conn: Mutex::new(conn),
        // stats: Mutex::default(),
    });
    Ok(data)
}

pub(crate) fn write_db(
    data: &MyData,
    cache: &HashMap<PathBuf, CacheEntry>,
) -> Result<(), rusqlite::Error> {
    let filter = |(_, entry): &(&PathBuf, &CacheEntry)| entry.new;
    println!(
        "Saving {}/{} cached thumbnails...",
        cache.iter().filter(filter).count(),
        cache.len()
    );

    let mut db = data.conn.lock().unwrap();

    let tx = db.transaction()?;

    for (key, value) in cache.iter().filter(filter) {
        let path_str = key.as_os_str();
        match &value.payload {
            CachePayload::File(payload) => {
                let byte_contents: &[u8] = &payload.data;
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
            CachePayload::Album(value) => {
                todo!()
            }
        }
    }
    tx.commit()?;
    Ok(())
}

fn table_exists(conn: &Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?1",
        [name],
        |row| row.get(0) as rusqlite::Result<String>,
    )
    .is_ok()
}
