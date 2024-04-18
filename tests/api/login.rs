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
    let status = response.status().as_u16();

    let response_html = response.text().await.expect("Failed to read response body");

    // we are expecting a teapot
    assert_eq!(status, 418);
    // the immediate response should contain the error message from htmx
    assert!(response_html.contains("Invalid credentials"));

    // act - 2
    // load the /login page
    // the first load should include the error message in the html after a full reload (from the cookie)
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("Invalid credentials"));

    // act - 3
    // reload the /login page
    // the second load should not include the error message in the html, the cookie should have been unset
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("Invalid credentials"));
}
