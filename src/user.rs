use actix_web::{
    error,
    web::{self, Bytes},
    HttpRequest, Result,
};
use rusqlite::{params, Connection};
use serde::Deserialize;
use sha1::{Digest, Sha1};

use crate::{
    db_utils::table_exists,
    map_err,
    session::{find_session, find_session_mut, get_valid_session, get_valid_session_mut},
    MyData,
};

#[derive(Debug, Deserialize)]
struct CreateUserParams {
    password: String,
}

pub(crate) fn init_table(conn: &Connection) -> anyhow::Result<()> {
    if !table_exists(&conn, "user") {
        conn.execute(
            "CREATE TABLE user (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                password TEXT,
                is_admin BOOL NOT NULL
            )",
            [],
        )
        .unwrap();

        // Add the admin account without a password.
        conn.execute(
            r#"INSERT INTO user (name, is_admin) VALUES ("admin", TRUE)"#,
            [],
        )?;
        println!("table \"user\" created!");
    }
    Ok(())
}

#[actix_web::post("/users/{name}")]
pub(crate) async fn create_user(
    data: web::Data<MyData>,
    name: web::Path<String>,
    params: web::Json<CreateUserParams>,
) -> Result<String> {
    let conn = data.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO user (name, password, is_admin) VALUES (?1, ?2, FALSE)",
        params![name.as_ref(), Sha1::digest(&params.password).as_slice()],
    )
    .map_err(map_err)?;
    Ok("Ok".to_string())
}

#[actix_web::post("/users/{name}/login")]
pub(crate) async fn login_user(
    data: web::Data<MyData>,
    req: HttpRequest,
    name: web::Path<String>,
    passwd: Bytes,
) -> Result<&'static str> {
    let mut sessions = data.sessions.write().unwrap();
    let session = get_valid_session_mut(&req, &mut sessions)?;
    println!("Attempt logging in: {name:?}");
    let conn = data.conn.lock().unwrap();
    let (id, db_passwd, is_admin) = conn
        .query_row_and_then(
            "SELECT id, password, is_admin FROM user WHERE name = ?1",
            [name.into_inner()],
            |q| -> rusqlite::Result<(usize, Option<Vec<u8>>, bool)> {
                Ok((q.get(0)?, q.get(1)?, q.get(2)?))
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => error::ErrorBadRequest("User not found"),
            e => map_err(e),
        })?;
    if db_passwd
        .map(|db_passwd| db_passwd != Sha1::digest(passwd).as_slice())
        .unwrap_or(false)
    {
        // TODO: is it safe to respond that the user name exists?
        return Err(error::ErrorNotAcceptable("Incorrect password"));
    }
    session.user_id = Some(id);
    session.is_admin = is_admin;
    Ok("Ok")
}

#[actix_web::post("/set_password")]
pub(crate) async fn set_user_password(
    data: web::Data<MyData>,
    req: HttpRequest,
    passwd: Bytes,
) -> Result<&'static str> {
    let sessions = data.sessions.read().unwrap();
    let session = get_valid_session(&req, &sessions)?;
    let user_id = session
        .user_id
        .ok_or_else(|| error::ErrorBadRequest("Please login first"))?;
    let conn = data.conn.lock().map_err(map_err)?;
    conn.execute(
        "UPDATE user SET password = ?1 WHERE id = ?2",
        params![Sha1::digest(passwd).as_slice(), user_id],
    )
    .map_err(map_err)?;
    Ok("Ok")
}
