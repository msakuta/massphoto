use super::THUMBNAIL_SIZE;
use crate::{map_err, CacheEntry, MyData};
use actix_files::NamedFile;
use actix_web::{error, http::header::LastModified, web, HttpRequest, HttpResponse, Result};
use image::{io::Reader as ImageReader, ImageOutputFormat};

use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
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

    let result = |out, modified| {
        let mut builder = HttpResponse::Ok();
        builder.content_type("image/jpg");
        if let Some(modified) = unix_to_system_time(modified) {
            builder.insert_header(LastModified(modified.into()));
        }
        Ok(builder.body(out))
    };

    if let Some(entry) = data.cache.lock().map_err(map_err)?.get(&abs_path) {
        // Defaults true because some filesystems do not support file modified dates. I don't know such a
        // filesystem, but Rust documentation says so.
        if get_file_modified(&abs_path)
            .map(|date| date <= entry.modified)
            .unwrap_or(true)
        {
            return result(entry.data.clone(), entry.modified);
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
            desc: None,
            data: out.clone(),
        },
    );

    result(out, modified)
}

/// Return modified date in days since Unix epoch
pub(crate) fn get_file_modified(path: &Path) -> anyhow::Result<f64> {
    let meta = fs::metadata(path)?;
    let modified = meta.modified()?;
    let unix_time = modified.duration_since(SystemTime::UNIX_EPOCH)?;
    Ok(unix_time.as_millis() as f64 / 1000. / 3600. / 24.)
}

/// Return modified date from days since Unix epoch
pub(crate) fn unix_to_system_time(unix: f64) -> Option<SystemTime> {
    SystemTime::UNIX_EPOCH.checked_add(Duration::new((unix * 3600. * 24.) as u64, 0))
}

pub(crate) async fn get_image_comment(
    data: web::Data<MyData>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let path: PathBuf = req.match_info().get("file").unwrap().parse().unwrap();
    let abs_path = data.path.lock().map_err(map_err)?.join(&path);

    let cache = data.cache.lock().map_err(map_err)?;
    let Some(entry) = cache.get(&abs_path) else {
        return Err(error::ErrorNotFound("Entry not found"));
    };
    let Some(desc) = &entry.desc else {
        return Err(error::ErrorNotFound("Desc not found"));
    };

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(desc.clone()))
}

pub(crate) async fn set_image_comment(
    data: web::Data<MyData>,
    mut payload: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse> {
    use futures_util::stream::StreamExt;
    const MAX_SIZE: usize = 1024 * 100;
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    let desc = std::str::from_utf8(&body).unwrap();

    let path: PathBuf = req.match_info().get("file").unwrap().parse().unwrap();
    let abs_path = data.path.lock().map_err(map_err)?.join(&path);

    println!("Comment posted on {path:?} ({abs_path:?}): {desc}");

    let mut cache = data.cache.lock().map_err(map_err)?;

    let mut inserted = false;
    let entry = cache.entry(abs_path.clone()).or_insert_with(|| {
        inserted = true;
        CacheEntry {
            new: true,
            modified: 0.,
            desc: Some(desc.to_string()),
            data: vec![],
        }
    });
    entry.desc = Some(desc.to_string());

    let mut db = data.conn.lock().unwrap();

    let tx = db.transaction().map_err(map_err)?;

    let updated = if inserted {
        tx.execute(
            "INSERT INTO file (path, modified, desc) VALUES (?1, ?2, ?3)",
            rusqlite::params![abs_path.to_str(), entry.modified, entry.desc],
        )
        .map_err(map_err)?
    } else {
        tx.execute(
            "UPDATE file SET desc = ?2 WHERE path = ?1",
            rusqlite::params![abs_path.to_str(), entry.desc],
        )
        .map_err(map_err)?
    };

    tx.commit().map_err(map_err)?;

    println!("inserted: {inserted}, updated: {updated}");

    Ok(HttpResponse::Ok().content_type("text/plain").body("ok"))
}
