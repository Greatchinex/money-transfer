use actix_web::web::{get, scope, ServiceConfig};
use actix_web_lab::middleware::from_fn;

use crate::handlers::wallets::my_wallets;
use crate::middlewares::auth::auth_middleware;

pub fn wallet_route_group(conf: &mut ServiceConfig) {
    let scope = scope("/api/wallet").route(
        "/my-wallets",
        get().to(my_wallets).wrap(from_fn(auth_middleware)),
    );

    conf.service(scope);
}
