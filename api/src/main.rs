use actix_web::{App, HttpServer};

use api::config;

#[cfg(not(tarpaulin_include))]
#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(config))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
