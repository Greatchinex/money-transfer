use actix_web::web;

use crate::handlers::users::{login, signup};

pub fn user_config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api/user").service(signup).service(login);

    conf.service(scope);
}
