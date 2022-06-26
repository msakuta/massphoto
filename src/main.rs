use actix_web::{web, App, HttpResponse, HttpServer};
use clap::Parser;
use dunce::canonicalize;
use handlebars::Handlebars;
use rusqlite::Connection;
use serde_json::json;
use std::{include_str, path::PathBuf, sync::Mutex};

struct MyData {
    // home_path: PathBuf,
    path: Mutex<PathBuf>,
    // cache: Mutex<HashMap<PathBuf, CacheEntry>>,
    // conn: Mutex<Connection>,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let conn = Connection::open("sqliter.db").map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("rusqlite: {:?}", e))
    })?;

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

    let data = web::Data::new(MyData {
        // home_path: canonicalize(PathBuf::from(&args.path))?,
        path: Mutex::new(canonicalize(PathBuf::from(args.path))?),
        // cache: Mutex::new(HashMap::default()),
        // conn: Mutex::new(conn),
        // stats: Mutex::default(),
    });
    // let data_copy = data.clone();
    let result = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/", web::get().to(image_list))
    })
    .bind((args.host, args.port))?
    .run()
    .await;

    // let cache = data_copy.cache.lock().unwrap();

    // if *ekey != Encrypt3Key::default() {
    //     let mut stats = data_copy.stats.lock().unwrap();
    //     let time_save_db = Instant::now();
    //     (|| -> Result<(), rusqlite::Error> {
    //         let filter = |(_, entry): &(&PathBuf, &CacheEntry)| entry.new;
    //         println!(
    //             "Saving {}/{} cached thumbnails...",
    //             cache.iter().filter(filter).count(),
    //             cache.len()
    //         );

    //         let mut db = data_copy.conn.lock().unwrap();

    //         let tx = db.transaction()?;

    //         for (key, value) in cache.iter().filter(filter) {
    //             let path_str = key.as_os_str();
    //             let time_thumb_encrypt = Instant::now();
    //             let mut reader: &[u8] = &value.data;
    //             let byte_contents = encrypt3_fast(&*ekey, &mut reader);
    //             stats
    //                 .thumbnail_encrypt
    //                 .add(time_thumb_encrypt.elapsed().as_micros());
    //             if tx
    //                 .query_row(
    //                     "SELECT path FROM file WHERE path=?1",
    //                     [path_str.to_str()],
    //                     |row| row.get::<_, String>(0),
    //                 )
    //                 .is_ok()
    //             {
    //                 tx.execute(
    //                     "UPDATE file SET modified = ?2, data = ?3 WHERE path = ?1",
    //                     rusqlite::params![path_str.to_str(), value.modified, byte_contents],
    //                 )?;
    //             } else {
    //                 tx.execute(
    //                     "INSERT INTO file (path, modified, data) VALUES (?1, ?2, ?3)",
    //                     rusqlite::params![path_str.to_str(), value.modified, byte_contents],
    //                 )?;
    //             }
    //         }
    //         tx.commit()?;
    //         Ok(())
    //     })()
    //     .expect("Error in saving cache");
    //     println!(
    //         "time save db: {} ms",
    //         time_save_db.elapsed().as_micros() as f64 / 1000.
    //     );
    //     println!("======= Average stats ========");
    //     println!("file_size: {} kb", stats.file_size);
    //     println!("read_disk: {} ms", stats.read_disk);
    //     println!("decrypt_image: {} ms", stats.decrypt_image);
    //     println!("decode_image: {} ms", stats.decode_image);
    //     println!("thumbnail: {} ms", stats.thumbnail);
    //     println!("thumbnail_encode: {} ms", stats.thumbnail_encode);
    //     println!("thumbnail_encrypt: {} ms", stats.thumbnail_encrypt);
    // }

    result
}

async fn image_list(data: web::Data<MyData>) -> HttpResponse {
    let path = data.path.lock().unwrap();
    let reg = Handlebars::new();

    HttpResponse::Ok().content_type("text/html").body(
        reg.render_template(
            include_str!("../static/templates/index.html"),
            &json!({
                "path": *path,
            }),
        )
        .unwrap(),
    )
}
