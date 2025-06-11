use actix_web::{App, HttpServer};
use std::env;

use game_api::config;

#[cfg(not(tarpaulin_include))]
#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "5001".to_string())
        .parse()
        .expect("PORT must be a number");

    HttpServer::new(|| App::new().configure(config))
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
