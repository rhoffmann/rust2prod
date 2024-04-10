use crate::helpers::spawn_app;

#[tokio::test]
async fn the_header_contains_an_error_message_on_failure() {
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "email": "not_existing_email",
        "password": "not_existing_password",
    });

    let response = app.post_login(&login_body).await;

    // let response_html = response.text().await.expect("Failed to read response body");

    let cookies = response.cookies().collect::<Vec<_>>();
    let flash_cookie = cookies.iter().find(|c| c.name() == "_flash").unwrap();

    // we are expecting a teapot
    assert_eq!(flash_cookie.value(), "Invalid credentials");
    assert_eq!(response.status().as_u16(), 418);
    // assert!(response_html.contains("Invalid credentials"));
}

// TODO: add test to check if the cookie is set and the correct html is rendered when loading the login page again
