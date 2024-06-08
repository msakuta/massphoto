use super::{
    auth::{authorized_path, validate_path, CheckAuth},
    THUMBNAIL_SIZE,
};
use crate::{
    cache::{CacheEntry, CachePayload, FilePayload},
    files::load_cache::load_cache_single,
    map_err,
    session::find_session,
    MyData,
};
use actix_files::NamedFile;
use actix_web::{
    error,
    http::header::LastModified,
    web::{self, Bytes},
    HttpRequest, HttpResponse, Result,
};
use image::{io::Reader as ImageReader, ImageOutputFormat};

use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

#[actix_web::get("/files/{path:.*}")]
pub(crate) async fn get_file(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    req: HttpRequest,
) -> Result<NamedFile> {
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let root_dir = data.path.lock().map_err(map_err)?;
    let abs_path = root_dir.join(&*path);
    let cache = data.cache.lock().unwrap();
    authorized_path(&path, session, &cache, CheckAuth::Read)?;
    println!("Opening {:?}", abs_path);
    Ok(NamedFile::open(abs_path)?)
}

#[actix_web::delete("/files/{path:.*}")]
pub(crate) async fn delete_file(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    validate_path(&*path)?;
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let root_dir = data.path.lock().map_err(map_err)?;
    let abs_path = root_dir.join(&*path);
    let cache = data.cache.lock().unwrap();
    authorized_path(&path, session, &cache, CheckAuth::Ownership)?;
    println!("Deleting {:?}", abs_path);
    std::fs::remove_file(&*abs_path)?;
    Ok(HttpResponse::Ok().content_type("text/plain").body("Ok"))
}

#[actix_web::post("/files/{path:.*}/move")]
pub(crate) async fn move_file(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    dest: String,
    req: HttpRequest,
) -> Result<HttpResponse> {
    validate_path(&*path)?;
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let root_dir = data.path.lock().map_err(map_err)?;
    let abs_path = root_dir.join(&*path);
    let dest = Path::new(&dest);
    let dest_path = root_dir.join(
        &path
            .file_name()
            .map(|path| dest.join(path))
            .unwrap_or_else(|| PathBuf::from(dest)),
    );
    let cache = data.cache.lock().unwrap();
    authorized_path(&path, session, &cache, CheckAuth::Ownership)?;
    println!("Moving {abs_path:?} to {dest_path:?}");
    std::fs::rename(&*abs_path, dest_path)?;
    Ok(HttpResponse::Ok().content_type("text/plain").body("Ok"))
}

#[actix_web::get("/thumbs/{path:.*}")]
pub(crate) async fn get_file_thumb(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let result = |out, modified| {
        let mut builder = HttpResponse::Ok();
        builder.content_type("image/jpg");
        if let Some(modified) = unix_to_system_time(modified) {
            builder.insert_header(LastModified(modified.into()));
        }
        Ok(builder.body(out))
    };

    let abs_path;
    {
        static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        let sessions = data.sessions.read().map_err(map_err)?;
        let session = find_session(&req, &sessions);
        let root_dir = data.path.lock().map_err(map_err)?;
        abs_path = root_dir.join(&*path);
        let cache = data.cache.lock().map_err(map_err)?;
        authorized_path(&path, session, &cache, CheckAuth::Read)?;
        let start = START.get_or_init(|| std::time::Instant::now());
        println!(
            "[{:?}] [{:?}] Opening {:?}",
            std::thread::current().id(),
            start.elapsed(),
            path
        );

        if let Some(entry) = cache.get(&*path) {
            // Defaults true because some filesystems do not support file modified dates. I don't know such a
            // filesystem, but Rust documentation says so.
            if get_file_modified(&abs_path)
                .map(|date| date <= entry.modified)
                .unwrap_or(true)
            {
                if let CachePayload::File(payload) = &entry.payload {
                    let data =
                        load_cache_single(&data.conn.lock().unwrap(), &path).map_err(map_err)?;
                    if !data.is_empty() {
                        return result(data, entry.modified);
                    }
                    if !payload.data.is_empty() {
                        return result(payload.data.clone(), entry.modified);
                    }
                } else {
                    return Err(error::ErrorInternalServerError(
                        "Album does not have thumbnail",
                    ));
                }
            } else {
                println!("Found thumbnail cache in db, but it is older than the file")
            }
        }
        // Drop all mutex locks here before entering CPU intense processing
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

    let mut cache = data.cache.lock().map_err(map_err)?;
    cache.insert(
        path.into_inner(),
        CacheEntry {
            new: true,
            modified,
            desc: None,
            payload: CachePayload::File(FilePayload { data: out.clone() }),
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

#[actix_web::get("/desc/{file:.*}")]
pub(crate) async fn get_image_desc(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let cache = data.cache.lock().map_err(map_err)?;
    authorized_path(&path, session, &cache, CheckAuth::Read)?;
    let Some(entry) = cache.get(&*path) else {
        return Err(error::ErrorNotFound("Entry not found"));
    };
    let Some(desc) = &entry.desc else {
        return Err(error::ErrorNotFound("Desc not found"));
    };

    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body(desc.clone()))
}

#[actix_web::post("/desc/{file:.*}")]
pub(crate) async fn set_image_desc(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    bytes: Bytes,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let desc = std::str::from_utf8(&bytes).unwrap();

    let mut cache = data.cache.lock().map_err(map_err)?;
    authorized_path(&path, session, &cache, CheckAuth::Ownership)?;

    println!("Description updated on {path:?}: {desc}");

    let mut inserted = false;
    let entry = cache.entry(path.clone()).or_insert_with(|| {
        inserted = true;
        CacheEntry {
            new: true,
            modified: 0.,
            desc: Some(desc.to_string()),
            payload: CachePayload::File(FilePayload { data: vec![] }),
        }
    });
    entry.desc = Some(desc.to_string());

    let mut db = data.conn.lock().unwrap();

    let tx = db.transaction().map_err(map_err)?;

    let updated = if inserted {
        tx.execute(
            "INSERT INTO file (path, modified, desc) VALUES (?1, ?2, ?3)",
            rusqlite::params![path.to_str(), entry.modified, entry.desc],
        )
        .map_err(map_err)?
    } else {
        tx.execute(
            "UPDATE file SET desc = ?2 WHERE path = ?1",
            rusqlite::params![path.to_str(), entry.desc],
        )
        .map_err(map_err)?
    };

    tx.commit().map_err(map_err)?;

    println!("inserted: {inserted}, updated: {updated}");

    Ok(HttpResponse::Ok().content_type("text/plain").body("ok"))
}

#[actix_web::post("/upload/{file:.*}")]
pub(crate) async fn upload(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    bytes: Bytes,
    req: HttpRequest,
) -> Result<HttpResponse> {
    validate_path(&*path)?;
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let root_dir = data.path.lock().map_err(map_err)?;
    let abs_path = root_dir.join(&*path);
    let cache = data.cache.lock().unwrap();
    authorized_path(&path, session, &cache, CheckAuth::Ownership)?;
    println!("Uploading {:?}", abs_path);
    std::fs::write(abs_path, bytes)?;
    Ok(HttpResponse::Ok().content_type("text/plain").body("ok"))
}
