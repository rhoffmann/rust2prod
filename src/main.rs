use rust2prod::{configuration::get_configuration};
use rust2prod::startup::build;
use rust2prod::telemetry::{get_subscriber, init_subscriber_once};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // configure tracing
    let subscriber = get_subscriber("rust2prod".into(), "info".into(), std::io::stdout);
    init_subscriber_once(subscriber);

    // read configuration
    let configuration = get_configuration().expect("Failed to read configuration");
    let server = build(configuration).await?;

    // trick the borrow checker :troll:
    let configuration = get_configuration().unwrap();
    println!("running on http://{}:{}", configuration.application.host, configuration.application.port);

    server.await?;
    Ok(())
}
