use crate::helpers::spawn_app;

#[tokio::test]
async fn the_header_contains_an_error_message_on_failure() {
    // arrange
    let app = spawn_app().await;

    let login_body = serde_json::json!({
        "email": "not_existing_email",
        "password": "not_existing_password",
    });

    // act - 1
    // send the post, assert the response (htmx snippet) is 418 and the returned html contains the error message (for htmx)
    let response = app.post_login(&login_body).await;

    // let response_html = response.text().await.expect("Failed to read response body");
    // assert the cookie is set (will it invalidate immediately?)
    let cookies = response.cookies().collect::<Vec<_>>();
    let flash_cookie = cookies.iter().find(|c| c.name() == "_flash").unwrap();

    // we are expecting a teapot
    assert_eq!(flash_cookie.value(), "Invalid credentials");
    assert_eq!(response.status().as_u16(), 418);
    // assert!(response_html.contains("Invalid credentials"));

    // act - 2
    // load the /login page
    // assert the cookie is set (the flash cookie)
    // assert the error message is displayed

    // act - 3
    // reload the /login page
    // assert the cookie is not set
    // assert the error message is not displayed
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("Invalid credentials")); // last should not have error message
}
