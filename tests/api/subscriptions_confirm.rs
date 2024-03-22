use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn confirmation_without_token_is_rejected_with_400() {
    let app = spawn_app().await;
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_200_if_called() {
    let app = spawn_app().await;
    let body = "name=the%20boss&email=the_boss%40gmail.com";

    Mock::given(path("/emails"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    // get first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    // act
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    assert_eq!(response.status(), 200);
}
