use crate::helpers::spawn_app;

#[tokio::test]
async fn the_header_contains_an_error_message_on_failure() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "email": "not_existing_email",
        "password": "not_existing_password",
    });

    let response = app.post_login(&login_body).await;

    let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();

    assert_eq!(flash_cookie.value(), "Invalid credentials");

    // we are expecting a 418 status code
    assert_eq!(
        response.headers().get("error-message").unwrap(),
        "Invalid credentials"
    );
    assert_eq!(response.status().as_u16(), 418);
}
