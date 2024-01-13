//! Data models for cached data on memory from DB.

use std::{collections::HashMap, path::PathBuf};

use actix_web::{web, HttpResponse};

use crate::{map_err, MyData};

#[derive(Debug)]
pub(crate) struct FilePayload {
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct AlbumPayload {
    pub password_hash: String,
}

#[derive(Debug)]
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
    pub(crate) fn is_locked(&self) -> bool {
        match self.payload {
            CachePayload::Album(ref album) => !album.password_hash.is_empty(),
            _ => false,
        }
    }

    pub(crate) fn _password_hash(&self) -> Option<&str> {
        match self.payload {
            CachePayload::Album(ref album) => Some(&album.password_hash),
            _ => None,
        }
    }
}

/// Cached data from DB and also filesystem. It is kept in-memory and written back to disk on exit.
pub(crate) type CacheMap = HashMap<PathBuf, CacheEntry>;

pub(crate) async fn clear_cache(data: web::Data<MyData>) -> actix_web::Result<HttpResponse> {
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
