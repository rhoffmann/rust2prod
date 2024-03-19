use rust2prod::{configuration::get_configuration};
use rust2prod::startup::{Application};
use rust2prod::telemetry::{get_subscriber, init_subscriber_once};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // configure tracing
    let subscriber = get_subscriber("rust2prod".into(), "info".into(), std::io::stdout);
    init_subscriber_once(subscriber);

    // read configuration
    let configuration = get_configuration().expect("Failed to read configuration");
    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;

    Ok(())
}
