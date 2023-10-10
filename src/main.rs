use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use env_logger;
use log::{error, info, LevelFilter};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde_json::json;
use std::{env, process};

use routes::users::user_config;

pub mod dto;
pub mod entities;
pub mod handlers;
pub mod routes;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

async fn health_checker() -> impl Responder {
    HttpResponse::Ok()
        .json(json!({ "status": "success", "message": "Welcome to MONEY TRANSFER APP" }))
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().json(
        json!({ "status": "error", "message": "Oops! We can't find the url you are looking for" }),
    )
}

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info,actix_web=info")
    }

    dotenv().ok();
    env_logger::init();

    let port = env::var("PORT").expect("PORT is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let socket_address = format!("{host}:{port}");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let mut opt = ConnectOptions::new(database_url);
    opt.sqlx_logging(false)
        .sqlx_logging_level(LevelFilter::Info);

    let pool = match Database::connect(opt).await {
        Ok(pool) => {
            info!("Successfully connected to database!!!");
            pool
        }
        Err(err) => {
            error!("Failed to connect to database {:?}", err);
            process::exit(1)
        }
    };

    let app_state = AppState { db: pool };

    println!("Starting server on port {}", port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PATCH", "PUT", "DELETE"])
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(health_checker))
            .configure(user_config)
            .default_service(web::route().to(not_found))
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind(&socket_address)?
    .run()
    .await?;

    Ok(())
}
