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
) -> bool {
    if !cache_entry.is_locked() {
        return true;
    }
    let Some(session) = session else {
        return false;
    };
    if session.is_admin {
        return true;
    }
    if cache_entry
        .owner()
        .zip(session.user_id)
        .map(|(owner, user)| owner == user)
        .unwrap_or(false)
    {
        return true;
    }
    // When we want to know about only ownership, do not care about temporary authentication.
    !matches!(check_auth, CheckAuth::Ownership) && session.auth_dirs.contains(path)
}

/// Check all ancestors of an album path.
pub(crate) fn authorized_path(
    path: &Path,
    session: Option<&Session>,
    cache: &CacheMap,
    check_auth: CheckAuth,
) -> actix_web::Result<()> {
    for ancestor_path in path.ancestors() {
        let Some(entry) = cache.get(ancestor_path) else {
            // If the album is absent in the ancestry list, it is considered owned by the admin.
            if session.map(|s| !s.is_admin).unwrap_or(true)
                && matches!(check_auth, CheckAuth::Ownership)
            {
                return Err(error::ErrorForbidden(
                    "Owner is different from the current session user. Ask the administrator to give you the ownership of this album.",
                ));
            }
            continue;
        };
        if authorized(ancestor_path, &entry, session, check_auth) {
            continue;
        }
        return Err(error::ErrorForbidden(
            "Forbidden to access password protected file",
        ));
    }
    Ok(())
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
    let root_dir = data.path.lock().map_err(map_err)?;
    let cache = data.cache.lock().map_err(map_err)?;
    let abs_path = root_dir.join(path.as_ref());
    if !session.is_admin {
        authorized_path(&path, Some(session), &cache, CheckAuth::Read)?;
    }
    let owner = cache
        .get(&abs_path)
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
