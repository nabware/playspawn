use actix_web::{HttpRequest, Responder, get, web};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(root);
}

#[get("/")]
async fn root(_req: HttpRequest) -> impl Responder {
    "Welcome!"
}
