use crate::MyData;
use actix_web::{web, HttpResponse};
use handlebars::Handlebars;
use serde_json::json;
use std::{ffi::OsStr, fs, include_str, path::Path};

const THUMBNAIL_SIZE: u32 = 100;

enum Entry {
    Dir {
        path: String,
        image_first: String,
        file_count: usize,
    },
    File {
        path: String,
        label: String,
        video: bool,
        encrypted: bool,
    },
}

pub(crate) async fn image_list(data: web::Data<MyData>) -> HttpResponse {
    let path = data.path.lock().unwrap();
    let reg = Handlebars::new();
    let mut has_any_video = false;

    let (dirs, files) = fs::read_dir(&*path)
        .unwrap()
        .filter_map(|res| {
            res.map(|e| {
                let path = e.path();
                if path.is_dir() {
                    Some(Entry::Dir {
                        path: path.file_name()?.to_str()?.to_owned(),
                        image_first: image_first(&path),
                        file_count: file_count(&path),
                    })
                } else if let Some(os_str) = path.extension() {
                    let ext = os_str.to_ascii_lowercase();
                    if ext == "jpg" || ext == "png" {
                        Some(Entry::File {
                            path: format!("t/{}", path.file_name().unwrap().to_str().unwrap()),
                            label: path.file_name().unwrap().to_str().unwrap().to_owned(),
                            video: false,
                            encrypted: false,
                        })
                    } else if ext == "enc" {
                        Some(Entry::File {
                            path: format!("e/t/{}", path.file_name().unwrap().to_str().unwrap()),
                            label: path.file_name().unwrap().to_str().unwrap().to_owned(),
                            video: false,
                            encrypted: true,
                        })
                    } else {
                        let pathstr = path.to_str()?;
                        if has_extension_segments(pathstr, ".webm.e")
                            || has_extension_segments(pathstr, ".mp4.e")
                        {
                            has_any_video = true;
                            Some(Entry::File {
                                path: format!("e/{}", path.file_name().unwrap().to_str().unwrap()),
                                label: path.file_name().unwrap().to_str().unwrap().to_owned(),
                                video: true,
                                encrypted: true,
                            })
                        } else if has_extension_segments(pathstr, ".webm")
                            || has_extension_segments(pathstr, ".mp4")
                        {
                            has_any_video = true;
                            Some(Entry::File {
                                path: path.file_name().unwrap().to_str().unwrap().to_owned(),
                                label: path.file_name().unwrap().to_str().unwrap().to_owned(),
                                video: true,
                                encrypted: false,
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
                encrypted,
            }) = v
            {
                files.push(json!({
                    "path": path,
                    "basename": Path::new(&path).file_name().unwrap_or_else(|| OsStr::new("")).to_string_lossy(),
                    "label": label,
                    "video": video,
                    "encrypted": encrypted,
                }));
            }
            (dirs, files)
        });

    HttpResponse::Ok().content_type("text/html").body(
        reg.render_template(
            include_str!("../static/templates/index.html"),
            &json!({
                "path": *path,
                "THUMBNAIL_SIZE": THUMBNAIL_SIZE,
                "dir_count": dirs.len(),
                "file_count": files.len(),
                "dirs": dirs,
                "files": files,
            }),
        )
        .unwrap(),
    )
}

/// Standard's `Path` can be used for last segment of file extensions,
/// but it won't work if it consists of multiple segments, like ".webm.e"
/// or ".tar.gz".
/// This is not idiomatic, but we can just compare last bytes of known length
/// to check the extension if the path is UTF-8 encoded.
pub(crate) fn has_extension_segments(path: &str, extension: &str) -> bool {
    path[path.len() - extension.len()..] == *extension
}

pub(crate) fn file_count(path: &Path) -> usize {
    fs::read_dir(path)
        .unwrap()
        .filter_map(|res| Some(res.ok()?.file_type().ok()?.is_file()))
        .filter(|b| *b)
        .count()
}

pub(crate) fn image_first(path: &Path) -> String {
    fs::read_dir(path)
        .unwrap()
        .filter_map(|res| {
            let path = res.ok()?.path();
            if path.is_file() {
                path.file_name().unwrap().to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .next()
        .map(|s| {
            let ext_lc = Path::new(&s).extension().map(|s| s.to_ascii_lowercase());
            match ext_lc.as_ref().and_then(|s| s.to_str()) {
                Some("jpg") => format!(
                    "t/{}/{}",
                    path.file_name().unwrap().to_str().unwrap().to_owned(),
                    &s
                ),
                Some("png") => format!(
                    "t/{}/{}",
                    path.file_name().unwrap().to_str().unwrap().to_owned(),
                    &s,
                ),
                _ => "".to_owned(),
            }
        })
        .unwrap_or_else(|| "".to_owned())
}
