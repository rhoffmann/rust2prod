use reqwest::Url;
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
    // parse json
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // extract link from one request field
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = get_link(body["html"].as_str().unwrap());
    let mut confirmation_link = Url::parse(&raw_confirmation_link).expect("Failed to parse URL.");

    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    // rewrite url to include port
    confirmation_link.set_port(Some(app.port)).unwrap();

    // act
    let response = reqwest::get(confirmation_link).await.unwrap();

    assert_eq!(response.status(), 200);
}
