use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use actix_web::{
    cookie::{
        time::{Duration, OffsetDateTime},
        Cookie, SameSite,
    },
    error,
    web::{self, Bytes},
    HttpRequest, HttpResponse,
};

use crate::{cache::CachePayload, MyData};

#[derive(Debug)]
pub(crate) struct Session {
    pub user_id: Option<usize>,
    /// Should we query this every time?
    pub is_admin: bool,
    pub auth_dirs: HashSet<PathBuf>,
}

impl Session {
    fn new() -> Self {
        Self {
            user_id: None,
            is_admin: false,
            auth_dirs: HashSet::new(),
        }
    }
}

pub(crate) type Sessions = HashMap<String, Session>;

#[actix_web::get("/sessions")]
pub(crate) async fn create_session(data: web::Data<MyData>, req: HttpRequest) -> HttpResponse {
    let mut sessions = data.sessions.write().unwrap();
    if find_session(&req, &sessions).is_some() {
        return HttpResponse::Ok().body("Ok");
    }
    let next_id = sha256::digest(
        &std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time always exist since UNIX_EPOCH")
            .as_nanos()
            .to_le_bytes(),
    );
    sessions.insert(next_id.clone(), Session::new());

    let cookie = Cookie::build("massPhotoSessionId", next_id)
        .path("/")
        .expires(OffsetDateTime::now_utc().checked_add(Duration::DAY * 10))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(false)
        .finish();
    HttpResponse::Ok()
        // .header("Set-Cookie", cookie.to_string())
        .cookie(cookie)
        .body("Ok")
}

pub(crate) fn find_session<'a>(req: &HttpRequest, sessions: &'a Sessions) -> Option<&'a Session> {
    req.cookie("massPhotoSessionId")
        .and_then(|cookie| sessions.get(cookie.value()))
}

pub(crate) fn find_session_mut<'a>(
    req: &HttpRequest,
    sessions: &'a mut Sessions,
) -> Option<&'a mut Session> {
    req.cookie("massPhotoSessionId")
        .and_then(|cookie| sessions.get_mut(cookie.value()))
}

/// A convenience function that maps absent session into an error
pub(crate) fn get_valid_session<'a>(
    req: &HttpRequest,
    sessions: &'a Sessions,
) -> actix_web::Result<&'a Session> {
    find_session(&req, sessions)
        .ok_or_else(|| error::ErrorBadRequest("Session expired. Please reload the browser."))
}

pub(crate) fn get_valid_session_mut<'a>(
    req: &HttpRequest,
    sessions: &'a mut Sessions,
) -> actix_web::Result<&'a mut Session> {
    find_session_mut(&req, sessions)
        .ok_or_else(|| error::ErrorBadRequest("Session expired. Please reload the browser."))
}

#[actix_web::post("/albums/{file:.*}/auth")]
pub(crate) async fn authorize_album(
    path: web::Path<PathBuf>,
    data: web::Data<MyData>,
    req: HttpRequest,
    bytes: Bytes,
) -> actix_web::Result<String> {
    let mut sessions = data.sessions.write().unwrap();
    let Some(session) = find_session_mut(&req, &mut sessions) else {
        return Err(error::ErrorBadRequest(
            "Session was not found; create a new session",
        ));
    };

    let password = String::from_utf8(bytes.to_vec())
        .map_err(|e| error::ErrorBadRequest(format!("Password needs to be a UTF-8 string: {e}")))?;

    let cache = data.cache.lock().unwrap();
    let entry = cache
        .get(&*path)
        .ok_or_else(|| error::ErrorNotFound("Directory not found"))?;
    let CachePayload::Album(ref album) = entry.payload else {
        return Err(error::ErrorBadRequest("File cannot be locked"));
    };

    if album.password_hash == sha256::digest(password) {
        session.auth_dirs.insert(path.into_inner());
    } else {
        return Err(error::ErrorNotAcceptable("Incorrect Password"));
    }

    Ok("Ok".to_owned())
}
