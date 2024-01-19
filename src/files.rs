mod auth;
mod images;
mod load_cache;
mod scan_dir;

use self::scan_dir::{scan_dir, ScanDirResult};
use crate::{session::find_session, MyData};
use actix_web::{error, web, HttpRequest, HttpResponse};

use std::{include_str, path::PathBuf};

pub(crate) use self::{
    auth::{authorized, get_owner, set_album_lock, set_owner, CheckAuth},
    images::{get_file, get_file_thumb, get_image_desc, set_image_desc},
    load_cache::load_cache,
};

const THUMBNAIL_SIZE: u32 = 100;

pub(crate) async fn index() -> HttpResponse {
    let html = include_str!("../public/index.html");
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[cfg(debug_assertions)]
#[actix_web::get("/main.js")]
pub(crate) async fn code() -> actix_web::Result<HttpResponse> {
    use actix_web::error;

    Err(error::ErrorInternalServerError(
        "Not implemented. Use `npm run dev` to run the frontend dev server.",
    ))
}

#[cfg(not(debug_assertions))]
#[actix_web::get("/build/bundle.js")]
pub(crate) async fn code() -> &'static str {
    include_str!("../public/build/bundle.js")
}

#[actix_web::get("/global.css")]
pub(crate) async fn get_global_css() -> HttpResponse {
    #[cfg(not(debug_assertions))]
    {
        HttpResponse::Ok()
            .content_type("text/css")
            .body(include_str!("../public/global.css"))
    }
    #[cfg(debug_assertions)]
    {
        HttpResponse::NotFound().body("Not found")
    }
}

#[actix_web::get("/build/bundle.css")]
pub(crate) async fn get_bundle_css() -> HttpResponse {
    #[cfg(not(debug_assertions))]
    {
        HttpResponse::Ok()
            .content_type("text/css")
            .body(include_str!("../public/build/bundle.css"))
    }
    #[cfg(debug_assertions)]
    {
        HttpResponse::NotFound().body("Not found")
    }
}

#[actix_web::get("/file_list/")]
pub(crate) async fn get_file_list_root(
    data: web::Data<MyData>,
    req: HttpRequest,
) -> actix_web::Result<web::Json<ScanDirResult>> {
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let path = data.path.lock().unwrap();
    let cache = data.cache.lock().unwrap();

    let res = scan_dir(&path, &cache, &path, session)?;

    Ok(web::Json(res))
}

#[actix_web::get("/file_list/{path:.*}")]
pub(crate) async fn get_file_list(
    path: web::Path<PathBuf>,
    data: web::Data<MyData>,
    req: HttpRequest,
) -> actix_web::Result<web::Json<ScanDirResult>> {
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let path = path.into_inner();
    let root_path = data.path.lock().unwrap();
    let cache = data.cache.lock().unwrap();

    if cache
        .get(&path)
        .map(|entry| authorized(&path, entry, session, CheckAuth::Read).is_err())
        .unwrap_or(false)
    {
        println!("Album {path:?} is locked");
        return Err(error::ErrorForbidden(
            "Forbidden to access password protected album",
        ));
    }
    let abs_path = root_path.join(&path);
    let res = scan_dir(&root_path, &cache, &abs_path, session)?;

    println!("File list for {path:?}");

    Ok(web::Json(res))
}

/// Standard's `Path` can be used for last segment of file extensions,
/// but it won't work if it consists of multiple segments, like ".webm.e"
/// or ".tar.gz".
/// This is not idiomatic, but we can just compare last bytes of known length
/// to check the extension if the path is UTF-8 encoded.
pub(crate) fn has_extension_segments(path: &str, extension: &str) -> bool {
    if extension.len() < path.len() {
        // Because accessing by bytes does not always split the string at character border
        // in UTF-8, we want to compare by bytes. This works because UTF-8 is stateless.
        // Do not try this with other character encodings.
        path.as_bytes()[path.len() - extension.len()..] == *extension.as_bytes()
    } else {
        false
    }
}
