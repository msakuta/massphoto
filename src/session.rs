use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use actix_web::{web, HttpResponse, cookie::{Cookie, SameSite}};
use rusqlite::ffi::sqlite3_randomness;

use crate::MyData;

pub(crate) struct Session {
    auth_dirs: HashSet<PathBuf>,
}

impl Session {
    fn new() -> Self {
        Self {
            auth_dirs: HashSet::new(),
        }
    }
}

pub(crate) type Sessions = HashMap<String, Session>;

pub(crate) async fn create_session(data: web::Data<MyData>) -> HttpResponse {
    let mut sessions = data.sessions.lock().unwrap();
    let next_id = sessions.len().to_string();
    sessions.insert(next_id.clone(), Session::new());

    let mut cookie = Cookie::build("sessionId", next_id)
        //  .domain("www.rust-lang.org")
        //  .path("/")
        .http_only(true)
        .same_site(SameSite::None)
         .finish();
    HttpResponse::Ok().cookie(cookie).body("Ok")
}

pub(crate) async fn authorize_album(path: web::Path<PathBuf>, data: web::Data<MyData>) -> String {
    let mut sessions = data.sessions.lock().unwrap();
    "Ok".to_owned()
}