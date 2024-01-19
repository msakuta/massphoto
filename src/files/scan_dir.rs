use std::{ffi::OsStr, fs, path::Path};

use serde_json::{json, Value};

use crate::{
    cache::CacheMap,
    files::{file_count, image_first},
    session::Session,
};

use super::{auth::authorized_path, authorized, has_extension_segments, CheckAuth};

#[derive(serde::Serialize)]
pub(super) struct ScanDirResult {
    files: Vec<Value>,
    dirs: Vec<Value>,
    has_any_video: bool,
    owned: bool,
}

pub(super) fn scan_dir(
    root_path: &Path,
    cache: &CacheMap,
    path: &Path,
    session: Option<&Session>,
) -> std::io::Result<ScanDirResult> {
    let mut has_any_video = false;

    let mut dirs = vec![];
    let mut files = vec![];
    for res in fs::read_dir(&*path)? {
        let Ok(e) = res else {
            continue;
        };
        let path = e.path();
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(rel_path) = path.strip_prefix(root_path) else {
            continue;
        };
        if path.is_dir() {
            let locked = cache
                .get(rel_path)
                .map(|entry| authorized(&rel_path, entry, session, CheckAuth::Read).is_err())
                .unwrap_or(false);
            dirs.push(json!({
                "path": file_name,
                "image_first": image_first(&path).and_then(|image_path| {
                    image_path.file_name()?.to_str().map(|s| s.to_owned().replace("\\", "/"))
                }),
                "file_count": file_count(&path),
                "locked": locked
            }));
        } else if let Some(os_str) = path.extension() {
            // Ignore files without extensions
            let ext = os_str.to_ascii_lowercase();
            let video;
            if ext == "jpg" || ext == "png" {
                video = false;
            } else {
                let Some(pathstr) = path.to_str() else {
                    continue;
                };
                if has_extension_segments(pathstr, ".webm")
                    || has_extension_segments(pathstr, ".mp4")
                {
                    video = true;
                    has_any_video = true;
                } else {
                    continue;
                }
            }
            files.push(json!({
                "path": file_name,
                "basename": Path::new(&path).file_name().unwrap_or_else(|| OsStr::new("")).to_string_lossy(),
                "label": file_name,
                "video": video,
            }));
        }
    }

    let owned = path
        .strip_prefix(root_path)
        .map(|path| authorized_path(&path, session, cache, CheckAuth::Ownership).is_ok())
        .unwrap_or(false);

    Ok(ScanDirResult {
        dirs,
        files,
        has_any_video,
        owned,
    })
}
