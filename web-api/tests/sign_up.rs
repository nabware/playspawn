use actix_web::{App, test};

use web_api::config;

#[actix_web::test]
async fn successful_sign_up() {
    let app = test::init_service(App::new().configure(config)).await;

    let req = test::TestRequest::get().uri("/auth/sign-up").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    assert_eq!(body, "Welcome!");
}
