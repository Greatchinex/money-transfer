use actix_web::web::{get, post, scope, ServiceConfig};
use actix_web_lab::middleware::from_fn;

use crate::handlers::users::{login, me, signup};
use crate::middlewares::auth::auth_middleware;

pub fn user_route_group(conf: &mut ServiceConfig) {
    let scope = scope("/api/user")
        .route("/signup", post().to(signup))
        .route("/login", post().to(login))
        .route("/me", get().to(me).wrap(from_fn(auth_middleware)));

    conf.service(scope);
}
