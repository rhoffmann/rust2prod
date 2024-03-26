use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

// compose subscriber
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
// higher rank trait bound, means that the type can be converted to a writer
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // set up bunyan logger with tracing-subscriber
    // fall back to printing all spans if RUST_LOG is not set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(env_filter));

    // output formatted logs to stdout
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    // set up tracing subscriber and return it
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

// register once a subscriber as global default
pub fn init_subscriber_once(subscriber: impl Subscriber + Send + Sync) {
    // redirect all logs from the log crate to the tracing subscriber
    LogTracer::init().expect("Failed to set logger");

    // set subscriber as global default
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

// spawn_block with tracing attached to current span
pub fn spawn_blocking_with_tracing<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
