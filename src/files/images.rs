use super::THUMBNAIL_SIZE;
use crate::{
    cache::{AlbumPayload, CacheEntry, CacheMap, CachePayload, FilePayload},
    files::{authorized, load_cache::load_cache_single},
    map_err,
    session::{find_session, get_valid_session, Session},
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

pub(crate) async fn get_file(data: web::Data<MyData>, req: HttpRequest) -> Result<NamedFile> {
    let sessions = data.sessions.read().unwrap();
    let session = find_session(&req, &sessions);
    let path: PathBuf = req.match_info().get("file").unwrap().parse().unwrap();
    let root_dir = data.path.lock().map_err(map_err)?;
    let abs_path = root_dir.join(&path);
    let cache = data.cache.lock().unwrap();
    authorized_path(&path, &root_dir, session, &cache)?;
    println!("Opening {:?}", abs_path);
    Ok(NamedFile::open(abs_path)?)
}

pub(crate) async fn get_file_thumb(
    data: web::Data<MyData>,
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
        let path: PathBuf = req.match_info().get("file").unwrap().parse().unwrap();
        let root_dir = data.path.lock().map_err(map_err)?;
        abs_path = root_dir.join(&path);
        let cache = data.cache.lock().map_err(map_err)?;
        authorized_path(&path, &root_dir, session, &cache)?;
        let start = START.get_or_init(|| std::time::Instant::now());
        println!(
            "[{:?}] [{:?}] Opening {:?}",
            std::thread::current().id(),
            start.elapsed(),
            abs_path
        );

        if let Some(entry) = cache.get(&abs_path) {
            // Defaults true because some filesystems do not support file modified dates. I don't know such a
            // filesystem, but Rust documentation says so.
            if get_file_modified(&abs_path)
                .map(|date| date <= entry.modified)
                .unwrap_or(true)
            {
                if let CachePayload::File(payload) = &entry.payload {
                    let data = load_cache_single(&data.conn.lock().unwrap(), &abs_path)
                        .map_err(map_err)?;
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
        abs_path,
        CacheEntry {
            new: true,
            modified,
            desc: None,
            payload: CachePayload::File(FilePayload { data: out.clone() }),
        },
    );

    result(out, modified)
}

fn authorized_path(
    path: &Path,
    root_dir: &Path,
    session: Option<&Session>,
    cache: &CacheMap,
) -> actix_web::Result<()> {
    for seg in path.ancestors() {
        let ancestor_path = root_dir.join(seg);
        let Some(entry) = cache.get(&ancestor_path) else {
            continue;
        };
        if authorized(&ancestor_path, &entry, session) {
            continue;
        }
        return Err(error::ErrorForbidden(
            "Forbidden to access password protected file",
        ));
    }
    Ok(())
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
    bytes: Bytes,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let desc = std::str::from_utf8(&bytes).unwrap();

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
            payload: CachePayload::File(FilePayload { data: vec![] }),
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

pub(crate) async fn set_album_lock(
    data: web::Data<MyData>,
    req: HttpRequest,
    bytes: Bytes,
) -> Result<HttpResponse> {
    let sessions = data.sessions.read().map_err(map_err)?;
    let session = get_valid_session(&req, &sessions)?;
    let path: PathBuf = req.match_info().get("file").unwrap().parse().unwrap();
    let root_dir = data.path.lock().map_err(map_err)?;
    let mut cache = data.cache.lock().map_err(map_err)?;
    if !session.is_admin {
        authorized_path(&path, &root_dir, Some(session), &cache)?;
    }
    let password = bytes.as_ref();
    let hash = if password.is_empty() {
        "".to_string()
    } else {
        sha256::digest(password)
    };

    let abs_path = root_dir.join(&path);

    println!("Password hash set on {path:?} ({abs_path:?}): {hash:?}");

    let mut inserted = false;
    let entry = cache.entry(abs_path.clone()).or_insert_with(|| {
        inserted = true;
        CacheEntry {
            new: true,
            modified: 0.,
            desc: None,
            payload: CachePayload::Album(AlbumPayload {
                password_hash: String::new(),
            }),
        }
    });

    let mut db = data.conn.lock().unwrap();

    let tx = db.transaction().map_err(map_err)?;

    let CachePayload::Album(ref mut payload) = entry.payload else {
        return Err(error::ErrorInternalServerError(
            "Logic error: inserted was not an album",
        ));
    };
    payload.password_hash = hash;
    let updated = if inserted {
        tx.execute(
            "INSERT INTO album (path, desc, password) VALUES (?1, ?2, ?3)",
            rusqlite::params![abs_path.to_str(), entry.desc, payload.password_hash],
        )
        .map_err(map_err)?
    } else {
        tx.execute(
            "UPDATE album SET password = ?2 WHERE path = ?1",
            rusqlite::params![abs_path.to_str(), payload.password_hash],
        )
        .map_err(map_err)?
    };

    tx.commit().map_err(map_err)?;

    println!("inserted: {inserted}, updated: {updated}");

    Ok(HttpResponse::Ok().content_type("text/plain").body("ok"))
}
