use once_cell::sync::Lazy;
use uuid::Uuid;
use rust2prod::telemetry::{get_subscriber, init_subscriber_once};
use rust2prod::configuration::{get_configuration, DatabaseSettings};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use rust2prod::startup::{Application, get_connection_pool};


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

impl TestApplication {
    pub async fn post_subscriptions(&self, body: &str) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

/// Spin up instance of the application
/// and return the address e.g. (http://localhost:<PORT>)
pub async fn spawn_app() -> TestApplication {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");

    let address = format!("http://127.0.0.1:{}", application.port());

    // drop the spawned future handle
    let _ = tokio::spawn(application.run_until_stopped());

    TestApplication {
        address,
        connection_pool: get_connection_pool(&configuration.database),
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
