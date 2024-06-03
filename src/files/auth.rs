//! Authentication related methods, i.e. involves both the file cache and the user accounts.

use crate::{
    cache::{CacheEntry, CacheMap, CachePayload},
    map_err,
    session::{get_valid_session, Session},
    MyData,
};

use actix_web::{
    error,
    web::{self, Bytes},
    HttpRequest, HttpResponse, Result,
};

use serde::Deserialize;

use std::path::{Path, PathBuf};

/// The modes to check authority. Explicitly defined to avoid bool-ish API.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CheckAuth {
    /// Check for the ownership
    Ownership,
    /// Check for read access
    Read,
}

/// Returns true when the path is accessible
pub(crate) fn authorized(
    path: &Path,
    cache_entry: &CacheEntry,
    session: Option<&Session>,
    check_auth: CheckAuth,
) -> actix_web::Result<()> {
    if !cache_entry.is_locked() {
        return Ok(());
    }
    let session = session
        .ok_or_else(|| error::ErrorForbidden("Session is invalid. Try reloading the browser"))?;
    if session.is_admin {
        return Ok(());
    }
    if cache_entry
        .owner()
        .zip(session.user_id)
        .map(|(owner, user)| owner == user)
        .unwrap_or(false)
    {
        return Ok(());
    }
    // When we want to know about only ownership, do not care about temporary authentication.
    if matches!(check_auth, CheckAuth::Ownership) {
        Err(error::ErrorForbidden(
            "Owner is different from the current session user. Ask the administrator to give you the ownership of this album.",
        ))
    } else if !session.auth_dirs.contains(path) {
        Err(error::ErrorForbidden("Not authorized to access"))
    } else {
        Ok(())
    }
}

/// Return containing directory if the path was a file. Return itself if the path was a directory
fn get_containing_directory<'a>(
    path: &'a Path,
    cache: &'a CacheMap,
) -> Option<(&'a Path, &'a CacheEntry)> {
    cache.get(path).and_then(|entry| match entry.payload {
        CachePayload::File(_) => path
            .parent()
            .and_then(|p| get_containing_directory(p, cache)),
        CachePayload::Album(_) => Some((path, entry)),
    })
}

/// Check if the path is valid (i.e. a valid string and does not contain "..")
pub(crate) fn validate_path(path: &Path) -> actix_web::Result<()> {
    let Some(path_str) = path.as_os_str().to_str() else {
        return Err(error::ErrorInternalServerError(
            "File path is not a valid string",
        ));
    };
    if path_str.contains("..") {
        return Err(error::ErrorInternalServerError(
            "Uploading to paths containing \"..\" is prohibited",
        ));
    }
    Ok(())
}

/// Check all ancestors of an album path.
pub(crate) fn authorized_path(
    path: &Path,
    session: Option<&Session>,
    cache: &CacheMap,
    check_auth: CheckAuth,
) -> actix_web::Result<()> {
    let entry = get_containing_directory(path, cache);
    // Our app authenticate per album. Check the containing album authentication.
    let Some((parent, entry)) = entry else {
        // Files in the root directory are considered owned by the admin.
        if session.map(|s| !s.is_admin).unwrap_or(false)
            && matches!(check_auth, CheckAuth::Ownership)
        {
            return Err(error::ErrorForbidden(
        "Owner is different from the current session user. Ask the administrator to give you the ownership of this album.",
            ));
        }
        return Ok(());
    };
    let res = authorized(parent, &entry, session, check_auth);
    res
}

#[actix_web::post("/albums/{file:.*}/lock")]
pub(crate) async fn set_album_lock(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    req: HttpRequest,
    bytes: Bytes,
) -> Result<HttpResponse> {
    let sessions = data.sessions.read().map_err(map_err)?;
    let session = get_valid_session(&req, &sessions)?;
    let user_id = session
        .user_id
        .ok_or_else(|| error::ErrorBadRequest("You need to login to lock an album"))?;
    let mut cache = data.cache.lock().map_err(map_err)?;
    if !session.is_admin {
        authorized_path(&path, Some(session), &cache, CheckAuth::Ownership)?;
    }
    let password = bytes.as_ref();
    let hash = if password.is_empty() {
        "".to_string()
    } else {
        sha256::digest(password)
    };

    println!("Password hash set on {path:?}: {hash:?}");

    let mut inserted = false;
    let entry = cache.entry(path.clone()).or_insert_with(|| {
        inserted = true;
        CacheEntry::album_with_owner(user_id)
    });

    let db = data.conn.lock().unwrap();

    let CachePayload::Album(ref mut payload) = entry.payload else {
        return Err(error::ErrorInternalServerError(
            "Logic error: inserted was not an album",
        ));
    };
    payload.password_hash = hash;
    let updated = if inserted {
        db.execute(
            "INSERT INTO album (path, desc, password, owner) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![path.to_str(), entry.desc, payload.password_hash, user_id],
        )
        .map_err(map_err)?
    } else {
        db.execute(
            "UPDATE album SET password = ?2 WHERE path = ?1",
            rusqlite::params![path.to_str(), payload.password_hash],
        )
        .map_err(map_err)?
    };

    println!("set_album_lock inserted: {inserted}, updated: {updated}");

    Ok(HttpResponse::Ok().content_type("text/plain").body("ok"))
}

#[actix_web::get("/albums/{path:.*}/owner")]
pub(crate) async fn get_owner(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    req: HttpRequest,
) -> Result<String> {
    let sessions = data.sessions.read().unwrap();
    let session = get_valid_session(&req, &sessions)?;
    let cache = data.cache.lock().map_err(map_err)?;
    if !session.is_admin {
        authorized_path(&path, Some(session), &cache, CheckAuth::Read)?;
    }
    let owner = cache
        .get(&*path)
        .and_then(|entry| entry.owner())
        .unwrap_or(1);
    Ok(owner.to_string())
}

#[derive(Deserialize)]
struct ChangeOwnerParams {
    user_id: usize,
}

#[actix_web::post("/albums/{path:.*}/set_owner")]
pub(crate) async fn set_owner(
    data: web::Data<MyData>,
    path: web::Path<PathBuf>,
    params: web::Json<ChangeOwnerParams>,
    req: HttpRequest,
) -> Result<&'static str> {
    let sessions = data.sessions.read().unwrap();
    let session = get_valid_session(&req, &sessions)?;
    if !session.is_admin {
        return Err(error::ErrorForbidden(
            "Only the admin is allowed to change owner",
        ));
    }
    let user_id = params.user_id;
    let mut cache = data.cache.lock().map_err(map_err)?;

    let mut inserted = false;
    let entry = cache.entry(path.clone()).or_insert_with(|| {
        inserted = true;
        CacheEntry::album_with_owner(user_id)
    });
    if let CachePayload::Album(ref mut payload) = entry.payload {
        payload.owner = user_id;
    }

    let conn = data.conn.lock().unwrap();

    let updated = if inserted {
        conn.execute(
            "INSERT INTO album (path, desc, password, owner) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![path.to_str(), entry.desc, entry.password_hash(), user_id],
        )
        .map_err(map_err)?
    } else {
        conn.execute(
            "UPDATE album SET owner = ?1 WHERE path = ?2",
            rusqlite::params![user_id, path.to_str()],
        )
        .map_err(map_err)?
    };

    println!("set_owner inserted: {inserted}, updated: {updated}");

    Ok("Ok")
}
