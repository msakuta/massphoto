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
use sha1::{Digest, Sha1};

use crate::{cache::CachePayload, map_err, MyData};

pub(crate) struct Session {
    pub user_id: Option<usize>,
    pub auth_dirs: HashSet<PathBuf>,
}

impl Session {
    fn new() -> Self {
        Self {
            user_id: None,
            auth_dirs: HashSet::new(),
        }
    }
}

pub(crate) type Sessions = HashMap<String, Session>;

pub(crate) async fn create_session(data: web::Data<MyData>, req: HttpRequest) -> HttpResponse {
    let mut sessions = data.sessions.write().unwrap();
    if find_session(&req, &sessions).is_some() {
        return HttpResponse::Ok().body("Ok");
    }
    let next_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time always exist since UNIX_EPOCH")
        .as_secs_f64()
        .to_string();
    sessions.insert(next_id.clone(), Session::new());

    let cookie = Cookie::build("masaPhotoSessionId", next_id)
        .domain("localhost")
        .path("/")
        .expires(OffsetDateTime::now_utc().checked_add(Duration::DAY))
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
    // if let Some(cookie) = req.headers().get("Cookie") {
    //     println!("Got Cookie header! {:?}", cookie.to_str());
    // }
    // let cookies = req.cookies();
    // println!("Cookies: {cookies:?}");
    // if let Ok(cookies) = cookies {
    //     for (i, cookie) in cookies.iter().enumerate() {
    //         println!("  Cookie[{i}]: {cookie:?}");
    //     }
    // }
    req.cookie("masaPhotoSessionId")
        .and_then(|cookie| sessions.get(cookie.value()))
}

pub(crate) fn find_session_mut<'a>(
    req: &HttpRequest,
    sessions: &'a mut Sessions,
) -> Option<&'a mut Session> {
    req.cookie("masaPhotoSessionId")
        .and_then(|cookie| sessions.get_mut(cookie.value()))
}

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
    let abs_path = data.path.lock().map_err(map_err)?.join(path.as_path());

    let password = String::from_utf8(bytes.to_vec())
        .map_err(|e| error::ErrorBadRequest(format!("Password needs to be a UTF-8 string: {e}")))?;

    let cache = data.cache.lock().unwrap();
    let entry = cache
        .get(&abs_path)
        .ok_or_else(|| error::ErrorNotFound("Directory not found"))?;
    let CachePayload::Album(ref album) = entry.payload else {
        return Err(error::ErrorBadRequest("File cannot be locked"));
    };

    if album.password_hash == Sha1::digest(password).as_slice() {
        session.auth_dirs.insert(abs_path);
    } else {
        return Err(error::ErrorNotAcceptable("Incorrect Password"));
    }

    Ok("Ok".to_owned())
}
