use once_cell::sync::Lazy;
use reqwest::Url;
use rust2prod::configuration::{get_configuration, DatabaseSettings};
use rust2prod::startup::{get_connection_pool, Application};
use rust2prod::telemetry::{get_subscriber, init_subscriber_once};
use sha3::Digest;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

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
    pub port: u16,
    pub connection_pool: PgPool,
    pub email_server: MockServer,
    pub test_user: TestUser,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApplication {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .post(&format!("{}/newsletters", &self.address))
            .header("Content-Type", "application/json")
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .body(body.to_string())
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        // parse json
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        // extract link from one request field
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = Url::parse(&raw_link).expect("Failed to parse URL.");
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(body["html"].as_str().unwrap());
        let plain_text = get_link(body["text"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    // pub async fn test_user(&self) -> (String, String) {
    //     let row = sqlx::query!("SELECT username, password_hash FROM users LIMIT 1")
    //         .fetch_one(&self.connection_pool)
    //         .await
    //         .expect("Failed to fetch test user from database.");

    //     (row.username, row.password)
    // }
}

/// Spin up instance of the application
/// and return the address e.g. (http://localhost:<PORT>)
pub async fn spawn_app() -> TestApplication {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");

    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application.port());

    // drop the spawned future handle
    let _ = tokio::spawn(application.run_until_stopped());

    let test_app = TestApplication {
        address,
        connection_pool: get_connection_pool(&configuration.database),
        email_server,
        port: application_port,
        test_user: TestUser::generate(),
    };

    test_app.test_user.store(&test_app.connection_pool).await;

    test_app
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

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let password_hash = sha3::Sha3_256::digest(self.password.as_bytes());
        let password_hash = format!("{:x}", password_hash);

        sqlx::query!(
            "INSERT INTO users (user_id, username, password_hash)
            VALUES ($1, $2, $3)",
            self.user_id,
            self.username,
            password_hash
        )
        .execute(pool)
        .await
        .expect("Failed to insert test user into database.");
    }
}
