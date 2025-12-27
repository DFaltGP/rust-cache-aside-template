mod core;
mod utils;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use dotenvy::dotenv;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::marker::PhantomData;

use crate::core::middlewares::auth::AuthMiddlewareFactory;

async fn health_check(pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query("SELECT 1").execute(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().body("api_core is alive and database is connected! 🚚"),
        Err(_) => HttpResponse::InternalServerError().body("Database connection failed"),
    }
}

// Apenas para fins de teste do middleware
async fn middleware_test() -> impl Responder {
    HttpResponse::Ok().body("The service is protected")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is missing on .env file");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed when trying to connect to database");

    log::info!("Database connection stablished successfully!");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health_check))
            .route("/auth/signup", web::post().to(core::auth::handler::signup))
            .route("/auth/login", web::post().to(core::auth::handler::login))
            .service(
                web::scope("/api/v1")
                    .wrap(AuthMiddlewareFactory {
                        _service: PhantomData,
                    })
                    .route("/protected", web::get().to(middleware_test)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
