use actix_web::web::{post, scope, ServiceConfig};

use crate::handlers::webhooks::paystack_webhook;

pub fn webhook_route_group(conf: &mut ServiceConfig) {
    let scope = scope("/api/webhook").route("/paystack", post().to(paystack_webhook));

    conf.service(scope);
}
