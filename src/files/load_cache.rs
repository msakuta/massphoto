use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Instant,
};

use rusqlite::Connection;

use crate::CacheEntry;

pub(crate) fn load_cache(
    cache: &mut HashMap<PathBuf, CacheEntry>,
    conn: &Connection,
    abs_path: &Path,
) -> rusqlite::Result<()> {
    #[derive(Debug)]
    struct File {
        path: String,
        modified: f64,
        data: Vec<u8>,
    }

    let time_load = Instant::now();

    println!("Searching files in {:?}", abs_path);

    let abs_path = if let Some(abs_path) = abs_path.to_str() {
        abs_path
    } else {
        return Ok(());
    };

    let mut stmt = conn.prepare("SELECT path, modified, data FROM file WHERE path LIKE ?1")?;
    let file_iter = stmt.query_map([format!("{}%", abs_path)], |row| {
        Ok(File {
            path: row.get(0)?,
            modified: row.get(1)?,
            data: row.get(2)?,
        })
    })?;

    for file in file_iter {
        let file = file?;
        cache.insert(
            PathBuf::from(file.path),
            CacheEntry {
                new: false,
                modified: file.modified,
                data: file.data,
            },
        );
    }
    println!(
        "Loaded cache db {} entries in {} ms",
        cache.len(),
        time_load.elapsed().as_micros() as f64 / 1e6
    );
    Ok(())
}
