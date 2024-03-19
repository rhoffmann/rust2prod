use std::net::TcpListener;
use once_cell::sync::Lazy;
use uuid::Uuid;
use rust2prod::email_client::EmailClient;
use rust2prod::telemetry::{get_subscriber, init_subscriber_once};
use rust2prod::configuration::{get_configuration, DatabaseSettings};
use sqlx::{Connection, Executor, PgConnection, PgPool};


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

pub struct TestApplication {
    pub address: String,
    pub connection_pool: PgPool,
}


/// Spin up instance of the application
/// and return the address e.g. (http://localhost:<PORT>)
pub async fn spawn_app() -> TestApplication {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let socket = listener.local_addr().unwrap();
    let address = format!("http://{}:{}", socket.ip(), socket.port());

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let sender_email = configuration.email_client.sender().expect("Invalid sender email address");

    let timeout = configuration.email_client.timeout();

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let connection_pool = configure_database(&configuration.database).await;

    let server =
        rust2prod::startup::run(listener, connection_pool.clone(), email_client).expect("Failed to bind address");

    // launch server as background task
    // drop the spawned future handle
    let _ = tokio::spawn(server);

    TestApplication {
        address,
        connection_pool,
    }
}


async fn configure_database(config: &DatabaseSettings) -> PgPool {
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
