use crate::auth::UserClaims;
use crate::result::ApiResult;
use crate::result::Error;
use crate::{auth, AppState};
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{post, web};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize)]
pub struct UnauthorizedUserData {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct NewUserResponse {
    id: i64,
    jwt: String,
}

//curl --request POST --url http://localhost:8080/api/user/create --header 'Content-Type: application/json' --data '{"username": "TestUser","password": "TestPassword"}'
#[post("/api/user/create")]
pub async fn user_create(
    new_user: Json<UnauthorizedUserData>,
    state: web::Data<AppState>,
) -> Result<ApiResult<NewUserResponse>, Error> {
    let users = sqlx::query!(
        "SELECT COUNT(*) as count FROM user WHERE username = ?",
        new_user.username
    )
    .fetch_all(&state.pool)
    .await?;

    if users[0].count > 0 {
        return Ok(ApiResult::error(
            StatusCode::BAD_REQUEST,
            "Username already taken",
        ));
    }

    // TODO: validate username and password
    let hash = auth::hash_password(&new_user.password)?;

    let mut conn = state.pool.acquire().await?;
    let id = sqlx::query!(
        "INSERT INTO user (username, password) VALUES (?, ?)",
        new_user.username,
        hash
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();
    Ok(ApiResult::data(
        StatusCode::CREATED,
        NewUserResponse {
            id,
            jwt: auth::create_jwt(id)?,
        },
    ))
}

#[derive(Serialize)]
pub struct UserLoginResponse {
    jwt: String,
}

//curl --request POST --url http://localhost:8080/api/user/login --header 'Content-Type: application/json' --data '{"username": "TestUser","password": "TestPassword"}'
#[post("/api/user/login")]
pub async fn user_login(
    user_data: Json<UnauthorizedUserData>,
    state: web::Data<AppState>,
) -> Result<ApiResult<UserLoginResponse>, Error> {
    let password_query = sqlx::query!(
        "SELECT id,password FROM user WHERE username = ?",
        user_data.username
    )
    .fetch_all(&state.pool)
    .await?;

    if password_query.is_empty()
        || !auth::verify_hash(&password_query[0].password, &user_data.password)
    {
        return Ok(ApiResult::error(
            StatusCode::BAD_REQUEST,
            "Invalid username or password",
        ));
    }

    let jwt = auth::create_jwt(password_query[0].id)?;
    Ok(ApiResult::data(StatusCode::OK, UserLoginResponse { jwt }))
}

#[derive(Serialize)]
pub struct UserAuthTestResponse {
    uid: i64,
    username: String,
}

#[post("/api/user/auth_test")]
pub async fn user_auth_test(
    user_claims: UserClaims,
    state: web::Data<AppState>,
) -> Result<ApiResult<UserAuthTestResponse>, Error> {
    let username_query = sqlx::query!("SELECT username FROM user WHERE id = ?", user_claims.uid)
        .fetch_all(&state.pool)
        .await?;
    if username_query.is_empty() {
        return Ok(ApiResult::error(StatusCode::NOT_FOUND, "User not found"));
    }
    let username = username_query[0].username.clone();

    Ok(ApiResult::data(
        StatusCode::OK,
        UserAuthTestResponse {
            uid: user_claims.uid,
            username,
        },
    ))
}

#[derive(Deserialize)]
pub struct UserPasswordChangeData {
    current_password: String,
    new_password: String,
}

#[post("/api/user/change_password")]
pub async fn user_change_password(
    user_claims: UserClaims,
    state: web::Data<AppState>,
    data: Json<UserPasswordChangeData>,
) -> Result<ApiResult<String>, Error> {
    let hash_query = sqlx::query!("SELECT password FROM user WHERE id = ?", user_claims.uid)
        .fetch_all(&state.pool)
        .await?;
    if hash_query.is_empty() {
        return Ok(ApiResult::error(StatusCode::NOT_FOUND, "User not founed"));
    }

    let saved_hash = hash_query[0].password.clone();
    if !auth::verify_hash(&saved_hash, &data.current_password) {
        return Ok(ApiResult::error(
            StatusCode::BAD_REQUEST,
            "Invalid password",
        ));
    }
    let new_hash = auth::hash_password(&data.new_password)?;

    sqlx::query!(
        "UPDATE user SET password = ? WHERE id = ?",
        new_hash,
        user_claims.uid
    )
    .execute(&mut state.pool.acquire().await?)
    .await?;

    Ok(ApiResult::ok())
}
