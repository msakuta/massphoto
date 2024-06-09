//! Data models for cached data on memory from DB.

use std::{collections::HashMap, path::PathBuf};

use actix_web::{error, web, HttpRequest, HttpResponse};

use crate::{map_err, session::get_valid_session, MyData};

#[derive(Debug, Clone)]
pub(crate) struct FilePayload {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub(crate) struct AlbumPayload {
    pub password_hash: String,
    pub owner: usize,
}

#[derive(Debug, Clone)]
pub(crate) enum CachePayload {
    File(FilePayload),
    Album(AlbumPayload),
}

#[derive(Debug)]
pub(crate) struct CacheEntry {
    pub new: bool,
    pub modified: f64,
    pub desc: Option<String>,
    pub payload: CachePayload,
}

impl CacheEntry {
    pub(crate) fn album_with_owner(owner: usize) -> Self {
        Self {
            new: true,
            modified: 0.,
            desc: None,
            payload: CachePayload::Album(AlbumPayload {
                password_hash: String::new(),
                owner,
            }),
        }
    }

    pub(crate) fn is_locked(&self) -> bool {
        match self.payload {
            CachePayload::Album(ref album) => !album.password_hash.is_empty(),
            _ => false,
        }
    }

    pub(crate) fn owner(&self) -> Option<usize> {
        match self.payload {
            CachePayload::Album(ref album) => Some(album.owner),
            _ => None,
        }
    }

    pub(crate) fn password_hash(&self) -> Option<&str> {
        match self.payload {
            CachePayload::Album(ref album) => Some(&album.password_hash),
            _ => None,
        }
    }
}

/// Cached data from DB and also filesystem. It is kept in-memory and written back to disk on exit.
pub(crate) type CacheMap = HashMap<PathBuf, CacheEntry>;

#[actix_web::get("/clear_cache")]
pub(crate) async fn clear_cache(
    data: web::Data<MyData>,
    req: HttpRequest,
) -> actix_web::Result<HttpResponse> {
    let sessions = data.sessions.read().unwrap();
    let session = get_valid_session(&req, &sessions)?;
    if !session.is_admin {
        return Err(error::ErrorForbidden(
            "Only admin is allowed to trigger clear cache",
        ));
    }
    let start = std::time::Instant::now();
    let mut cache = data.cache.lock().map_err(map_err)?;
    for (_key, entry) in cache.iter_mut() {
        match entry.payload {
            CachePayload::File(ref mut f) => f.data.clear(),
            _ => {}
        }
    }
    println!(
        "Cleared {} cache entries in {}s",
        cache.len(),
        start.elapsed().as_secs_f64() / 1e3
    );
    let conn = data.conn.lock().map_err(map_err)?;
    conn.execute("UPDATE file SET data = x''", [])
        .map_err(map_err)?;
    Ok(HttpResponse::Ok().body("Ok"))
}
