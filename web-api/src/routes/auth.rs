use actix_web::{HttpRequest, Responder, get, web};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").service(sign_up));
}

#[get("/sign-up")]
async fn sign_up(_req: HttpRequest) -> impl Responder {
    "Welcome!"
}
