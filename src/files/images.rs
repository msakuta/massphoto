use super::THUMBNAIL_SIZE;
use crate::{map_err, CacheEntry, MyData};
use actix_files::NamedFile;
use actix_web::{error, web, HttpRequest, HttpResponse, Result};
use image::{io::Reader as ImageReader, ImageOutputFormat};

use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    time::SystemTime,
};

pub(crate) async fn get_file(data: web::Data<MyData>, req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().get("file").unwrap().parse().unwrap();
    let abs_path = data
        .path
        .lock()
        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?
        .join(path);
    println!("Opening {:?}", abs_path);
    Ok(NamedFile::open(abs_path)?)
}

pub(crate) async fn get_file_thumb(
    data: web::Data<MyData>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let path: PathBuf = req.match_info().get("file").unwrap().parse().unwrap();
    let abs_path = data
        .path
        .lock()
        .map_err(|e| error::ErrorInternalServerError(e.to_string()))?
        .join(path);
    println!("Opening {:?}", abs_path);

    if let Some(entry) = data.cache.lock().map_err(map_err)?.get(&abs_path) {
        // Defaults true because some filesystems do not support file modified dates. I don't know such a
        // filesystem, but Rust documentation says so.
        if get_file_modified(&abs_path)
            .map(|date| date <= entry.modified)
            .unwrap_or(true)
        {
            return Ok(HttpResponse::Ok()
                .content_type("image/jpg")
                .body(entry.data.clone()));
        } else {
            println!("Found thumbnail cache in db, but it is older than the file")
        }
    }

    let img = ImageReader::open(&abs_path)?
        .decode()
        .map_err(|err| error::ErrorInternalServerError(err.to_string()))?;
    let thumbnail = img.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
    let mut out = vec![];
    thumbnail
        .write_to(&mut Cursor::new(&mut out), ImageOutputFormat::Jpeg(85))
        .map_err(|err| error::ErrorInternalServerError(err.to_string()))?;

    let modified = get_file_modified(&abs_path).unwrap_or(0.);

    data.cache.lock().map_err(map_err)?.insert(
        abs_path,
        CacheEntry {
            new: true,
            modified,
            data: out.clone(),
        },
    );

    Ok(HttpResponse::Ok().content_type("image/jpg").body(out))
}

/// Return modified date in days since Unix epoch
pub(crate) fn get_file_modified(path: &Path) -> anyhow::Result<f64> {
    let meta = fs::metadata(path)?;
    let modified = meta.modified()?;
    let unix_time = modified.duration_since(SystemTime::UNIX_EPOCH)?;
    Ok(unix_time.as_millis() as f64 / 1000. / 3600. / 24.)
}
