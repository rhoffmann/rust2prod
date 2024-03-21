use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_200_successful_for_valid_data() {
    // arrange
    let app = spawn_app().await;
    let body = "name=the%20boss&email=the_boss%40gmail.com";

    Mock::given(path("/emails"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // act
    let response = app.post_subscriptions(body.into()).await;

    // assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "the_boss@gmail.com");
    assert_eq!(saved.name, "the boss");
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    // arrange
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=the%20boss&email=notanemail", "invalid email"),
        ("name=&email=the_boss%40gmail.com", "empty name"),
        ("name=the%20boss&email=", "empty email"),
    ];

    for (body, error_message) in test_cases {
        // act
        let response = app.post_subscriptions(body.into()).await;

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not return 400 Bad Request when payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_data() {
    // arrange
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=the%20boss", "missing the email"),
        ("email=the_boss%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    // act

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 with payload {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_for_valid_data() {
    // arrange
    let app = spawn_app().await;
    let body = "name=the%20boss&email=the_boss%40gmail.com";

    Mock::given(path("/emails"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // act
    app.post_subscriptions(body.into()).await;
    // assert on drop
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // arrange
    let app = spawn_app().await;
    let body = "name=the%20boss&email=the_boss%40gmail.com";

    Mock::given(path("/emails"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // act
    app.post_subscriptions(body.into()).await;

    // assert

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

    let html_link = get_link(body["html"].as_str().unwrap());
    let plain_text_link = get_link(body["text"].as_str().unwrap());

    assert_eq!(html_link, plain_text_link);
}
