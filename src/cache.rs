//! Data models for cached data on memory from DB.

use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub(crate) struct FilePayload {
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct AlbumPayload {
    pub password_hash: Vec<u8>,
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

    pub(crate) fn password_hash(&self) -> Option<&[u8]> {
        match self.payload {
            CachePayload::Album(ref album) => Some(&album.password_hash),
            _ => None,
        }
    }
}

/// Cached data from DB and also filesystem. It is kept in-memory and written back to disk on exit.
pub(crate) type CacheMap = HashMap<PathBuf, CacheEntry>;
