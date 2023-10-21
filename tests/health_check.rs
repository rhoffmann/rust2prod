use rust2prod::configuration::get_configuration;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

/// Spin up instance of the application
/// and return the address e.g. (http://localhost:<PORT>)
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let socket = listener.local_addr().unwrap();
    let port = socket.port();
    let ip = socket.ip().to_string();

    let server = rust2prod::startup::run(listener).expect("Failed to bind address");

    // launch server as background task
    // drop the spawned future handle
    let _ = tokio::spawn(server);

    format!("http://{}:{}", ip, port)
}

/// --- arrange, act, assert

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_successful_for_valid_data() {
    // arrange
    let address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    // requires 'Connection' trait from sqlx
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres.");

    let client = reqwest::Client::new();

    // act
    let body = "name=the%20boss&email=the_boss%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "the_boss@gmail.com");
    assert_eq!(saved.name, "the boss");
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_data() {
    // arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=the%20boss", "missing the email"),
        ("email=the_boss%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    // act

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not fail with 400 with payload {}.",
            error_message
        );
    }
}
