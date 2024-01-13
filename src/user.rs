use actix_web::{
    error,
    web::{self, Bytes},
    HttpRequest, Result,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::{
    db_utils::table_exists,
    map_err,
    session::{get_valid_session, get_valid_session_mut},
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

#[derive(Serialize)]
struct ListElementUser {
    id: usize,
    name: String,
    password: bool,
}

#[actix_web::get("/users")]
pub(crate) async fn list_users(
    data: web::Data<MyData>,
    req: HttpRequest,
) -> Result<web::Json<Vec<ListElementUser>>> {
    let sessions = data.sessions.read().unwrap();
    let session = get_valid_session(&req, &sessions)?;
    if !session.is_admin {
        return Err(error::ErrorForbidden("Non-admin cannot list users"));
    }
    let conn = data.conn.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, password is not null and length(password) != 0, is_admin FROM user",
        )
        .map_err(map_err)?;
    let users: Vec<_> = stmt
        .query_map([], |row| {
            let id: usize = row.get(0)?;
            let name: String = row.get(1)?;
            Ok(ListElementUser {
                id,
                name,
                password: row.get(2)?,
            })
        })
        .map_err(map_err)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?;
    Ok(web::Json(users))
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
        params![name.as_ref(), sha256::digest(&params.password)],
    )
    .map_err(map_err)?;
    Ok("Ok".to_string())
}

#[actix_web::delete("/users/{id}")]
pub(crate) async fn delete_user(
    data: web::Data<MyData>,
    id: web::Path<usize>,
    req: HttpRequest,
) -> Result<String> {
    let sessions = data.sessions.read().unwrap();
    let session = get_valid_session(&req, &sessions)?;
    if !session.is_admin {
        return Err(error::ErrorForbidden("Only the admin can delete a user"));
    }
    if session.user_id == Some(*id) {
        return Err(error::ErrorBadRequest("You cannot delete yourself"));
    }
    let conn = data.conn.lock().unwrap();
    conn.execute("DELETE FROM user WHERE id = ?1", params![id.as_ref()])
        .map_err(map_err)?;
    Ok("Ok".to_string())
}

#[derive(Serialize)]
struct StatusUserResult {
    logged_in: bool,
    is_admin: bool,
    name: Option<String>,
}

#[actix_web::get("/user_status")]
pub(crate) async fn status_user(
    data: web::Data<MyData>,
    req: HttpRequest,
) -> Result<web::Json<StatusUserResult>> {
    let sessions = data.sessions.read().unwrap();
    let session = get_valid_session(&req, &sessions)?;
    let Some(user_id) = session.user_id else {
        return Ok(web::Json(StatusUserResult {
            logged_in: false,
            is_admin: false,
            name: None,
        }));
    };
    let conn = data.conn.lock().unwrap();
    let (name, is_admin) = conn
        .query_row_and_then(
            "SELECT name, is_admin FROM user WHERE id = ?1",
            [user_id],
            |q| -> rusqlite::Result<(String, bool)> { Ok((q.get(0)?, q.get(1)?)) },
        )
        .map_err(map_err)?;
    Ok(web::Json(StatusUserResult {
        logged_in: true,
        is_admin,
        name: Some(name),
    }))
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
            |q| -> rusqlite::Result<(usize, Option<String>, bool)> {
                Ok((q.get(0)?, q.get(1)?, q.get(2)?))
            },
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => error::ErrorBadRequest("User not found"),
            e => map_err(e),
        })?;
    if db_passwd
        .map(|db_passwd| db_passwd != sha256::digest(passwd.as_ref()))
        .unwrap_or(false)
    {
        // TODO: is it safe to respond that the user name exists?
        return Err(error::ErrorNotAcceptable("Incorrect password"));
    }
    session.user_id = Some(id);
    session.is_admin = is_admin;
    Ok("Ok")
}

#[actix_web::post("/user_logout")]
pub(crate) async fn logout_user(data: web::Data<MyData>, req: HttpRequest) -> Result<&'static str> {
    let mut sessions = data.sessions.write().unwrap();
    let session = get_valid_session_mut(&req, &mut sessions)?;
    println!("Attempt logging out: {:?}", session.user_id);
    if session.user_id.is_none() {
        return Err(error::ErrorBadRequest("Already logged out"));
    }
    session.user_id = None;
    session.is_admin = false;
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
        params![sha256::digest(passwd.as_ref()), user_id],
    )
    .map_err(map_err)?;
    Ok("Ok")
}
