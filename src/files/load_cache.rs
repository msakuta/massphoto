use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Instant,
};

use rusqlite::Connection;

use crate::cache::{AlbumPayload, CacheEntry, CachePayload, FilePayload};

pub(crate) fn load_cache(
    cache: &mut HashMap<PathBuf, CacheEntry>,
    conn: &Connection,
    abs_path: &Path,
) -> rusqlite::Result<()> {
    #[derive(Debug)]
    struct File {
        path: String,
        modified: f64,
        desc: Option<String>,
        // data: Vec<u8>,
    }

    let time_load = Instant::now();

    println!("Searching files in {:?}", abs_path);

    let abs_path = if let Some(abs_path) = abs_path.to_str() {
        abs_path
    } else {
        return Ok(());
    };

    let mut stmt = conn.prepare("SELECT path, modified, desc FROM file WHERE path LIKE ?1")?;
    let file_iter = stmt.query_map([format!("{}%", abs_path)], |row| {
        Ok(File {
            path: row.get(0)?,
            modified: row.get(1)?,
            desc: row.get(2).ok(),
            // data: row.get(3)?,
        })
    })?;

    for file in file_iter {
        let file = file?;
        cache.insert(
            PathBuf::from(file.path),
            CacheEntry {
                new: false,
                modified: file.modified,
                desc: file.desc,
                payload: CachePayload::File(FilePayload { data: vec![] }),
            },
        );
    }

    #[derive(Debug)]
    struct Album {
        path: String,
        modified: f64,
        desc: Option<String>,
        password: String,
    }

    let mut stmt = conn.prepare("SELECT path, desc, password FROM album WHERE path LIKE ?1")?;
    let album_iter = stmt.query_map([format!("{}%", abs_path)], |row| {
        let bytes: Vec<u8> = row.get(2)?;
        let s = String::from_utf8(bytes).unwrap();
        Ok(Album {
            path: row.get(0)?,
            modified: 0.,
            desc: row.get(1).ok(),
            password: s,
        })
    })?;

    for album in album_iter {
        let album = album?;
        cache.insert(
            PathBuf::from(album.path),
            CacheEntry {
                new: false,
                modified: album.modified,
                desc: album.desc,
                payload: CachePayload::Album(AlbumPayload {
                    password_hash: album.password,
                }),
            },
        );
    }
    println!(
        "Loaded cache db {} entries in {} s",
        cache.len(),
        time_load.elapsed().as_micros() as f64 / 1e6
    );
    Ok(())
}

pub(crate) fn load_cache_single(conn: &Connection, path: &Path) -> anyhow::Result<Vec<u8>> {
    let mut stmt = conn.prepare("SELECT data FROM file WHERE path = ?1")?;
    let mut rows = stmt.query([path.to_str().unwrap()])?;
    Ok(rows
        .next()?
        .ok_or_else(|| anyhow::anyhow!("Specified path entry was not found on the db"))?
        .get(0)?)
}
