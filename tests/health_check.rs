use rust2prod::configuration::{get_configuration, DatabaseSettings};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use once_cell::sync::Lazy;
use uuid::Uuid;
use rust2prod::telemetry::{get_subscriber, init_subscriber_once};

pub struct TestApplication {
    pub address: String,
    pub connection_pool: PgPool,
}


static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber_once(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber_once(subscriber);
    }
});

/// Spin up instance of the application
/// and return the address e.g. (http://localhost:<PORT>)
async fn spawn_app() -> TestApplication {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let socket = listener.local_addr().unwrap();
    let address = format!("http://{}:{}", socket.ip(), socket.port());

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server =
        rust2prod::startup::run(listener, connection_pool.clone()).expect("Failed to bind address");

    // launch server as background task
    // drop the spawned future handle
    let _ = tokio::spawn(server);

    TestApplication {
        address,
        connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");

    connection_pool
}

/// --- arrange, act, assert

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_successful_for_valid_data() {
    // arrange
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    // act
    let body = "name=the%20boss&email=the_boss%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

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
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=the%20boss&email=notanemail", "invalid email"),
        ("name=&email=the_boss%40gmail.com", "empty name"),
        ("name=the%20boss&email=", "empty email"),
    ];

    for (body, error_message) in test_cases {
        // act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

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

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=the%20boss", "missing the email"),
        ("email=the_boss%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    // act

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
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
