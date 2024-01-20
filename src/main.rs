mod cache;
mod db_utils;
mod files;
mod session;
mod user;

use crate::{
    cache::{clear_cache, CacheMap},
    db_utils::{init_db, periodic_cleanup, write_db},
    files::{
        code, get_bundle_css, get_file, get_file_list, get_file_list_root, get_file_thumb,
        get_global_css, get_image_desc, get_owner, index, set_album_lock, set_image_desc,
        set_owner, upload,
    },
    session::{authorize_album, create_session, Sessions},
    user::{
        create_user, delete_user, list_users, login_user, logout_user, set_user_password,
        status_user,
    },
};
use actix_cors::Cors;
use actix_web::{error, web, App, Error, HttpServer};
use clap::Parser;

use rusqlite::Connection;
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, RwLock},
    time::Instant,
};

/// The global state of the server. Mutable shared states shall be wrapped in a mutex.
struct MyData {
    /// The root path of the photoalbum
    path: Mutex<PathBuf>,
    cache: Mutex<CacheMap>,
    conn: Mutex<Connection>,
    // stats: Mutex<StatsBundle>,
    sessions: RwLock<Sessions>,
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(default_value = ".")]
    path: String,
    #[clap(
        short,
        long,
        default_value = "8808",
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
    #[clap(
        short,
        long,
        default_value = "http://localhost:8808",
        help = "The allowed Access-Control-Allow-Origin value. Set \"*\" to allow any origin."
    )]
    cors_origin: String,
    #[clap(
        short = 'P',
        long,
        default_value = "120",
        help = "Interval to auto-cleanup cache memory, in seconds."
    )]
    cleanup_period: u64,
}

fn map_err(err: impl ToString) -> Error {
    error::ErrorInternalServerError(err.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

async fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    let data = init_db(Path::new(&args.path))?;

    let data_copy = data.clone();
    let server_fut = HttpServer::new(move || {
        #[cfg(not(debug_assertions))]
        let cors = {
            let mut cors = Cors::default();
            cors = if args.cors_origin == "*" {
                cors.allow_any_origin()
            } else {
                cors.allowed_origin(&args.cors_origin)
            };
            cors.allowed_methods(vec!["GET", "POST"])
                .allowed_header(actix_web::http::header::CONTENT_TYPE)
                .max_age(3600)
        };
        #[cfg(debug_assertions)]
        let cors = Cors::permissive();

        App::new()
            .app_data(data.clone())
            .wrap(cors)
            .route("/", web::get().to(index))
            .service(code)
            .service(get_file_list_root)
            .service(get_file_list)
            .service(get_global_css)
            .service(get_bundle_css)
            .service(list_users)
            .service(create_user)
            .service(delete_user)
            .service(status_user)
            .service(login_user)
            .service(logout_user)
            .service(set_user_password)
            .service(get_image_desc)
            .service(set_image_desc)
            .service(get_file_thumb)
            .service(get_file)
            .service(set_album_lock)
            .service(authorize_album)
            .service(get_owner)
            .service(set_owner)
            .service(create_session)
            .service(clear_cache)
            .service(upload)
    })
    .bind((args.host, args.port))?
    .run();

    actix_rt::spawn(periodic_cleanup(data_copy.clone(), args.cleanup_period));

    let result = server_fut.await;

    let mut cache = data_copy.cache.lock().unwrap();

    let time_save_db = Instant::now();
    write_db(&data_copy, &mut cache).expect("Error in saving cache");
    println!(
        "time save db: {} s",
        time_save_db.elapsed().as_micros() as f64 / 1e6
    );

    result.map_err(anyhow::Error::new)
}

fn measure_time<T>(f: impl FnOnce() -> T) -> (T, f64) {
    let start = std::time::Instant::now();
    let ret = f();
    (ret, start.elapsed().as_secs_f64())
}
