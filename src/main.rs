use actix_web::{web, App, HttpServer};

use sqlx::{Pool, Sqlite, SqlitePool};

mod auth;
mod result;
mod routes;

pub struct AppState {
    pool: Pool<Sqlite>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePool::connect("sqlite://data.db")
        .await
        .expect("failed to connect to database");
    let app_state = web::Data::new(AppState { pool });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(routes::user::user_create)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
