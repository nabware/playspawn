use actix_web::{App, test};

use game_api::config;

#[actix_web::test]
async fn smoke_test() {
    let _ = test::init_service(App::new().configure(config)).await;
}
