use std::net::TcpListener;

use rust2prod::{configuration::get_configuration, startup::run};
use sqlx::PgPool;
use rust2prod::email_client::EmailClient;
use rust2prod::telemetry::{get_subscriber, init_subscriber_once};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // configure tracing
    let subscriber = get_subscriber("rust2prod".into(), "info".into(), std::io::stdout);
    init_subscriber_once(subscriber);

    // application code
    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind port");

    let connection_pool = PgPool::connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration.email_client.sender().expect("Invalid sender email address");
    let timeout = configuration.email_client.timeout();

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    println!("running on http://{}", &address);

    // need to await here, because we return a Server from run
    run(listener, connection_pool, email_client)?.await
}
