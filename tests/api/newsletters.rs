use uuid::Uuid;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{spawn_app, ConfirmationLinks, TestApplication};

#[tokio::test]
async fn non_existing_user_is_rejected() {
    let app = spawn_app().await;

    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .basic_auth(&username, Some(&password))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "title": "newsletter title",
            "content": {
                "text": "Newsletter content",
                "html": "<h1>Newsletter content</h1>"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // random users should not pass the basic auth
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["www-authenticate"]
    );
}

#[tokio::test]
async fn invalid_password_is_rejected() {
    let app = spawn_app().await;
    let test_user = app.test_user;
    let password = Uuid::new_v4().to_string();

    assert_ne!(test_user.password, password);

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .basic_auth(&test_user.username, Some(&password))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "title": "newsletter title",
            "content": {
                "text": "Newsletter content",
                "html": "<h1>Newsletter content</h1>"
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["www-authenticate"]
    );
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // expect NO request is made to the email server
        .expect(0)
        .mount(&app.email_server)
        .await;

    // act
    let newsletter_request_body = serde_json::json!({
        "title": "newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<h1>Newsletter content</h1>"
        }
    });

    let response = app.post_newsletters(newsletter_request_body).await;

    // assert
    assert_eq!(200, response.status().as_u16());

    // mock verification on drop
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/emails"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // act
    let newsletter_request_body = serde_json::json!({
        "title": "newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<h1>Newsletter content</h1>"
        }
    });

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(200, response.status().as_u16());

    // remember mock is asserted on drop (that the request POST /emails was made successfully -> email has been sent)
}

#[tokio::test]
async fn newsletters_return_400_for_invalid_data() {
    let app = spawn_app().await;
    let test_cases = vec![
        (serde_json::json!({}), "missing title and content"),
        (serde_json::json!({"title": "title"}), "missing content"),
        (
            serde_json::json!({"content": {"text": "text"}}),
            "missing title",
        ),
        (
            serde_json::json!({"title": "title", "content": {}}),
            "empty content",
        ),
    ];

    for (test_case, error_message) in test_cases {
        let response = app.post_newsletters(test_case).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 with payload {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn request_missing_authorization_are_rejected() {
    let app = spawn_app().await;

    let newsletter_request_body = serde_json::json!({
        "title": "newsletter title",
        "content": {
            "text": "Newsletter content",
            "html": "<h1>Newsletter content</h1>"
        }
    });

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .header("Content-Type", "application/json")
        .body(newsletter_request_body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["www-authenticate"]
    );
}

async fn create_confirmed_subscriber(app: &TestApplication) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

// use application api to create a new subscriber
async fn create_unconfirmed_subscriber(app: &TestApplication) -> ConfirmationLinks {
    let body = "name=the%20boss&email=the_boss%40gmail.com";

    let _mock_guard = Mock::given(path("/emails"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(email_request)
}
