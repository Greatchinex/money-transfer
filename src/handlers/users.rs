use actix_web::{post, HttpResponse, Responder};
use serde_json::json;

#[post("/signup")]
pub async fn signup() -> impl Responder {
    HttpResponse::Created()
        .json(json!({ "status": "success", "message": "User created successfully" }))
}

#[post("/login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok().json(json!({ "status": "success", "message": "Login successful" }))
}
