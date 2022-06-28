mod files;

use crate::files::{
    get_file, get_file_list, get_file_list_root, get_file_thumb, index, load_cache,
};
use actix_cors::Cors;
use actix_web::{error, http, web, App, Error, HttpServer};
use clap::Parser;
use dunce::canonicalize;
use rusqlite::Connection;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};

struct CacheEntry {
    new: bool,
    modified: f64,
    data: Vec<u8>,
}

struct MyData {
    home_path: PathBuf,
    path: Mutex<PathBuf>,
    cache: Mutex<HashMap<PathBuf, CacheEntry>>,
    conn: Mutex<Connection>,
    // stats: Mutex<StatsBundle>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(default_value = ".")]
    path: String,
    #[clap(
        short,
        long,
        default_value = "8082",
        help = "The port number to listen to."
    )]
    port: u16,
    #[clap(
        short,
        long,
        default_value = "127.0.0.1",
        help = "The host address to listen to. By default, only the localhost can access."
    )]
    host: String,
}

fn map_err(err: impl ToString) -> Error {
    error::ErrorInternalServerError(err.to_string())
}

macro_rules! implement_static_bytes {
    ($func:ident, $path:literal) => {
        async fn $func() -> &'static [u8] {
            include_bytes!($path)
        }
    };
}

implement_static_bytes!(get_home_icon, "../assets/home.png");
implement_static_bytes!(get_up_icon, "../assets/up.png");
implement_static_bytes!(get_left_icon, "../assets/left.png");
implement_static_bytes!(get_right_icon, "../assets/right.png");

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

async fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    let conn = Connection::open("sqliter.db")?;

    if conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='file'",
            [],
            |row| row.get(0) as rusqlite::Result<String>,
        )
        .is_ok()
    {
        println!("table opened");
    } else {
        conn.execute(
            "CREATE table file (
            path TEXT PRIMARY KEY,
            modified REAL,
            data BLOB
        )",
            [],
        )
        .unwrap();
        println!("table created!");
    }

    let mut cache = HashMap::new();
    load_cache(&mut cache, &conn, &Path::new(&args.path))?;

    let data = web::Data::new(MyData {
        home_path: canonicalize(PathBuf::from(&args.path))?,
        path: Mutex::new(canonicalize(PathBuf::from(args.path))?),
        cache: Mutex::new(cache),
        conn: Mutex::new(conn),
        // stats: Mutex::default(),
    });
    let data_copy = data.clone();
    let result = HttpServer::new(move || {
        #[cfg(not(debug_assertions))]
        let cors = Cors::default()
            // .allowed_origin("http://localhost:8080/")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        #[cfg(debug_assertions)]
        let cors = Cors::permissive();

        App::new()
            .app_data(data.clone())
            .wrap(cors)
            .route("/", web::get().to(index))
            .route(
                "/main.js",
                web::get().to(|| async { include_str!("main.js") }),
            )
            .service(get_file_list_root)
            .service(get_file_list)
            .route("/thumbs/{file:.*}", web::get().to(get_file_thumb))
            .route("/files/{file:.*}", web::get().to(get_file))
            .route("/home.png", web::get().to(get_home_icon))
            .route("/up.png", web::get().to(get_up_icon))
            .route("/left.png", web::get().to(get_left_icon))
            .route("/right.png", web::get().to(get_right_icon))
    })
    .bind((args.host, args.port))?
    .run()
    .await;

    let cache = data_copy.cache.lock().unwrap();

    let time_save_db = Instant::now();
    (|| -> Result<(), rusqlite::Error> {
        let filter = |(_, entry): &(&PathBuf, &CacheEntry)| entry.new;
        println!(
            "Saving {}/{} cached thumbnails...",
            cache.iter().filter(filter).count(),
            cache.len()
        );

        let mut db = data_copy.conn.lock().unwrap();

        let tx = db.transaction()?;

        for (key, value) in cache.iter().filter(filter) {
            let path_str = key.as_os_str();
            let byte_contents: &[u8] = &value.data;
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
        tx.commit()?;
        Ok(())
    })()
    .expect("Error in saving cache");
    println!(
        "time save db: {} s",
        time_save_db.elapsed().as_micros() as f64 / 1e6
    );

    result.map_err(anyhow::Error::new)
}
