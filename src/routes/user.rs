use crate::result::ApiResult;
use crate::result::Error;
use crate::AppState;
use actix_web::http::StatusCode;
use actix_web::{post, web};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize)]
pub struct UserData {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct NewUserResponse {
    pub(crate) id: i64,
}

#[post("/api/user/create")]
pub async fn user_create(
    new_user: web::Json<UserData>,
    data: web::Data<AppState>,
) -> Result<ApiResult<NewUserResponse>, Error> {
    let users = sqlx::query!(
        "SELECT COUNT(*) as count FROM user WHERE username = ?",
        new_user.username
    )
    .fetch_all(&data.pool)
    .await?;

    if users[0].count > 0 {
        return Ok(ApiResult::error(
            StatusCode::BAD_REQUEST,
            "Username already taken",
        ));
    }

    // TODO: validate username and password
    let hash = crate::auth::hash_password(&new_user.password)?;

    let mut conn = data.pool.acquire().await?;
    let id = sqlx::query!(
        "INSERT INTO user (username, password) VALUES (?, ?)",
        new_user.username,
        hash
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();
    Ok(ApiResult::data(StatusCode::CREATED, NewUserResponse { id }))
}
