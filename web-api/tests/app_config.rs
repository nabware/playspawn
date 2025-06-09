use actix_web::{App, test};

use web_api::config;

#[actix_web::test]
async fn app_config() {
    let _ = test::init_service(App::new().configure(config)).await;
}
