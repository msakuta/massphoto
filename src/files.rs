mod images;
mod load_cache;

use crate::MyData;
use actix_web::{web, HttpResponse};
use handlebars::Handlebars;
use serde_json::{json, Value};
use std::{
    ffi::OsStr,
    fs, include_str,
    path::{Path, PathBuf},
};

pub(crate) use self::{
    images::{get_file, get_file_thumb},
    load_cache::load_cache,
};

const THUMBNAIL_SIZE: u32 = 100;

enum Entry {
    Dir {
        path: String,
        image_first: Option<String>,
        file_count: usize,
    },
    File {
        path: String,
        label: String,
        video: bool,
    },
}

fn scan_dir(path: &Path) -> (Vec<Value>, Vec<Value>, bool) {
    let mut has_any_video = false;

    let (dirs, files) = fs::read_dir(&*path)
    .unwrap()
    .filter_map(|res| {
        res.map(|e| {
            let path = e.path();
            if path.is_dir() {
                Some(Entry::Dir {
                    path: path.file_name()?.to_str()?.to_owned(),
                    image_first: image_first(&path).and_then(|path| path.to_str().map(|s| s.to_owned())),
                    file_count: file_count(&path),
                })
            } else if let Some(os_str) = path.extension() {
                let ext = os_str.to_ascii_lowercase();
                if ext == "jpg" || ext == "png" {
                    Some(Entry::File {
                        path: format!("{}", path.file_name()?.to_str()?),
                        label: path.file_name()?.to_str()?.to_owned(),
                        video: false,
                    })
                } else {
                    let pathstr = path.to_str()?;
                    if has_extension_segments(pathstr, ".webm")
                        || has_extension_segments(pathstr, ".mp4")
                    {
                        has_any_video = true;
                        Some(Entry::File {
                            path: path.file_name()?.to_str()?.to_owned(),
                            label: path.file_name()?.to_str()?.to_owned(),
                            video: true,
                        })
                    } else {
                        None
                    }
                }
            } else {
                // Ignore files without extensions
                None
            }
        })
        .ok()
    })
    .fold((vec![], vec![]), |(mut dirs, mut files), v| {
        if let Some(Entry::Dir { path, image_first, file_count }) = v {
            dirs.push(json!({
                "path": path,
                "image_first": image_first,
                "file_count": file_count
            }));
        } else if let Some(Entry::File {
            path,
            label,
            video,
        }) = v
        {
            files.push(json!({
                "path": path,
                "basename": Path::new(&path).file_name().unwrap_or_else(|| OsStr::new("")).to_string_lossy(),
                "label": label,
                "video": video,
            }));
        }
        (dirs, files)
    });

    (dirs, files, has_any_video)
}

pub(crate) async fn index(data: web::Data<MyData>) -> HttpResponse {
    let path = data.path.lock().unwrap();
    let reg = Handlebars::new();

    HttpResponse::Ok().content_type("text/html").body(
        reg.render_template(
            include_str!("../static/templates/index.html"),
            &json!({
                "THUMBNAIL_SIZE": THUMBNAIL_SIZE,
            }),
        )
        .unwrap(),
    )
}

#[actix_web::get("/file_list/")]
pub(crate) async fn get_file_list_root(data: web::Data<MyData>) -> HttpResponse {
    let path = data.path.lock().unwrap();

    let (dirs, files, _) = scan_dir(&path);

    HttpResponse::Ok()
        .content_type("application/json")
        .json(&json!({
            "path": "",
            "dirs": dirs,
            "files": files,
        }))
}

#[actix_web::get("/file_list/{path}")]
pub(crate) async fn get_file_list(
    path: web::Path<PathBuf>,
    data: web::Data<MyData>,
) -> HttpResponse {
    let path = path.into_inner();
    let abs_path = data.path.lock().unwrap().join(&path);

    let (dirs, files, _) = scan_dir(&abs_path);

    HttpResponse::Ok()
        .content_type("application/json")
        .json(&json!({
            "path": *path,
            "dirs": dirs,
            "files": files,
        }))
}

/// Standard's `Path` can be used for last segment of file extensions,
/// but it won't work if it consists of multiple segments, like ".webm.e"
/// or ".tar.gz".
/// This is not idiomatic, but we can just compare last bytes of known length
/// to check the extension if the path is UTF-8 encoded.
pub(crate) fn has_extension_segments(path: &str, extension: &str) -> bool {
    if extension.len() < path.len() {
        // Because accessing by bytes does not always split the string at character border
        // in UTF-8, we want to compare by bytes. This works because UTF-8 is stateless.
        // Do not try this with other character encodings.
        path.as_bytes()[path.len() - extension.len()..] == *extension.as_bytes()
    } else {
        false
    }
}

pub(crate) fn file_count(path: &Path) -> usize {
    fs::read_dir(path)
        .unwrap()
        .filter_map(|res| Some(res.ok()?.file_type().ok()?.is_file()))
        .filter(|b| *b)
        .count()
}

pub(crate) fn image_first(path: &Path) -> Option<PathBuf> {
    fs::read_dir(path)
        .unwrap()
        .filter_map(|res| res.ok())
        .find(|res| {
            let path = res.path();
            if path.is_file() {
                let ext_lc = path.extension().map(|s| s.to_ascii_lowercase());
                match ext_lc.as_ref().and_then(|s| s.to_str()) {
                    Some("jpg") | Some("png") => true,
                    _ => false,
                }
            } else {
                false
            }
        })
        .map(|entry| entry.path())
}

pub(crate) fn dir_list(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("No parent directory"))?;

    Ok(fs::read_dir(parent)
        .unwrap()
        .filter_map(|res| {
            res.map(|e| {
                let path = e.path();
                if path.is_dir() {
                    Some(path)
                } else {
                    None
                }
            })
            .ok()
            .flatten()
        })
        .collect())
}
