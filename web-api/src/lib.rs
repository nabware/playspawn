use actix_web::web;

mod routes;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(routes::auth::config);
}
