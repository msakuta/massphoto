use actix_web::{
    error,
    web::{self, Bytes},
    HttpRequest, Result,
};
use rusqlite::{params, Connection};
use serde::Deserialize;
use sha1::{Digest, Sha1};

use crate::{db_utils::table_exists, map_err, session::find_session_mut, MyData};

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
                password TEXT
            )",
            [],
        )
        .unwrap();

        // Add the admin account without a password.
        conn.execute(r#"INSERT INTO user (name) VALUES ("admin")"#, [])?;
        println!("table \"album\" created!");
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
        "INSERT INTO user (name, password) VALUES (?1, ?2)",
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
    let Some(session) = find_session_mut(&req, &mut sessions) else {
        return Err(error::ErrorBadRequest(
            "Session expired. Please reload the browser.",
        ));
    };
    println!("Attempt logging in: {name:?}");
    let conn = data.conn.lock().unwrap();
    let (id, db_passwd): (usize, Option<Vec<u8>>) = conn
        .query_row_and_then(
            "SELECT id, password FROM user WHERE name = ?1",
            [name.into_inner()],
            |q| -> rusqlite::Result<(usize, Option<Vec<u8>>)> { Ok((q.get(0)?, q.get(1)?)) },
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
    Ok("Ok")
}
