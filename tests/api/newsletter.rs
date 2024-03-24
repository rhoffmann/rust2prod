use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{spawn_app, ConfirmationLinks, TestApplication};

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

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", app.address))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execute request.");

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

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", app.address))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    // remember mock is asserted on drop (that the request POST /emails was made successfully -> email has been sent)
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
