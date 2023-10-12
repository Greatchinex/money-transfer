use actix_web::web;

use crate::handlers::users::{login, me, signup};

pub fn user_config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api/user")
        .service(signup)
        .service(login)
        .service(me);

    conf.service(scope);
}
