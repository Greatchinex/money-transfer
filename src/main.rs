use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use env_logger;
use sea_orm::{Database, DatabaseConnection};
use std::{env, process};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppState {
    db: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "money_transfer_logs")
    }

    dotenv().ok();
    env_logger::init();

    let port = env::var("PORT").expect("PORT is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let socket_address = format!("{host}:{port}");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    let pool = match Database::connect(&database_url).await {
        Ok(pool) => {
            println!("Successfully connected to database!!!");
            pool
        }
        Err(err) => {
            println!("Failed to connect to database {:?}", err);
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
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind(&socket_address)?
    .run()
    .await?;

    Ok(())
}
