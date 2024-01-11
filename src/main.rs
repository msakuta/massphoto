mod cache;
mod db_utils;
mod files;
mod session;

use crate::{
    cache::CacheMap,
    db_utils::{init_db, write_db},
    files::{
        code, get_bundle_css, get_file, get_file_list, get_file_list_root, get_file_thumb,
        get_global_css, get_image_comment, index, set_album_lock, set_image_comment,
    },
    session::{authorize_album, create_session, Sessions},
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
implement_static_bytes!(get_lock_icon, "../assets/lock.png");
implement_static_bytes!(get_directory_icon, "../assets/directory.png");
implement_static_bytes!(get_video_icon, "../assets/video.png");
implement_static_bytes!(get_close_icon, "../assets/close.png");
implement_static_bytes!(get_magnify_icon, "../assets/magnify.png");
implement_static_bytes!(get_minify_icon, "../assets/minify.png");
implement_static_bytes!(get_fit_icon, "../assets/fit.png");
implement_static_bytes!(get_comment_icon, "../assets/comment.png");
implement_static_bytes!(get_left_angle_icon, "../assets/leftAngle.png");
implement_static_bytes!(get_right_angle_icon, "../assets/rightAngle.png");
implement_static_bytes!(get_unknown_icon, "../assets/unknown.png");

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
    let result = HttpServer::new(move || {
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
            .route("/comments/{file:.*}", web::get().to(get_image_comment))
            .route("/comments/{file:.*}", web::post().to(set_image_comment))
            .route("/thumbs/{file:.*}", web::get().to(get_file_thumb))
            .route("/files/{file:.*}", web::get().to(get_file))
            .route("/albums/{file:.*}/lock", web::post().to(set_album_lock))
            .route("/albums/{file:.*}/auth", web::post().to(authorize_album))
            .route("/sessions", web::get().to(create_session))
            .route("/home.png", web::get().to(get_home_icon))
            .route("/up.png", web::get().to(get_up_icon))
            .route("/left.png", web::get().to(get_left_icon))
            .route("/right.png", web::get().to(get_right_icon))
            .route("/lock.png", web::get().to(get_lock_icon))
            .route("/directory.png", web::get().to(get_directory_icon))
            .route("/video.png", web::get().to(get_video_icon))
            .route("/close.png", web::get().to(get_close_icon))
            .route("/magnify.png", web::get().to(get_magnify_icon))
            .route("/minify.png", web::get().to(get_minify_icon))
            .route("/fit.png", web::get().to(get_fit_icon))
            .route("/comment.png", web::get().to(get_comment_icon))
            .route("/leftAngle.png", web::get().to(get_left_angle_icon))
            .route("/rightAngle.png", web::get().to(get_right_angle_icon))
            .route("/unknown.png", web::get().to(get_unknown_icon))
    })
    .bind((args.host, args.port))?
    .run()
    .await;

    let cache = data_copy.cache.lock().unwrap();

    let time_save_db = Instant::now();
    write_db(&data_copy, &cache).expect("Error in saving cache");
    println!(
        "time save db: {} s",
        time_save_db.elapsed().as_micros() as f64 / 1e6
    );

    result.map_err(anyhow::Error::new)
}
