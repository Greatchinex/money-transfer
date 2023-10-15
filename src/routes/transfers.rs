use actix_web::web::{post, scope, ServiceConfig};
use actix_web_lab::middleware::from_fn;

use crate::handlers::transfers::fund_account;
use crate::middlewares::auth::auth_middleware;

pub fn transfer_route_group(conf: &mut ServiceConfig) {
    let scope = scope("/api/transfer").route(
        "/fund-account",
        post().to(fund_account).wrap(from_fn(auth_middleware)),
    );

    conf.service(scope);
}
