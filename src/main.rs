use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde_json::json;
use std::{env, io, process};
use tracing::{error, info, log::LevelFilter};
use tracing_actix_web::TracingLogger;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use routes::transfers::transfer_route_group;
use routes::users::user_route_group;
use routes::wallets::wallet_route_group;
use routes::webhooks::webhook_route_group;

pub mod dto;
pub mod entities;
pub mod handlers;
pub mod middlewares;
pub mod routes;
pub mod utils;

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
    dotenv().ok();
    LogTracer::init().expect("Unable to setup log tracer");

    let app_name = env::var("APP_NAME").expect("APP_NAME is not set in .env file");
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(io::stdout());
    let bunyan_formatting_layer = BunyanFormattingLayer::new(app_name, non_blocking_writer);
    let subscriber = Registry::default()
        .with(EnvFilter::new("INFO"))
        .with(JsonStorageLayer)
        .with(bunyan_formatting_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");

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

    info!("Starting server on port {}", port);
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PATCH", "PUT", "DELETE"])
            .supports_credentials();

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(health_checker))
            .configure(user_route_group)
            .configure(wallet_route_group)
            .configure(transfer_route_group)
            .configure(webhook_route_group)
            .default_service(web::route().to(not_found))
            .wrap(cors)
            .wrap(TracingLogger::default())
    })
    .bind(&socket_address)?
    .run()
    .await?;

    Ok(())
}
