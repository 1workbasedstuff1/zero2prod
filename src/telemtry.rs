use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{
    EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt,
};

// creat subscriber for the logger
// 2 - want to add different "SINKS" for logging ie where that data is sent to
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // NOTE
    // takes RUST_LOG= ... value and uses it to create
    // a filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

    // Bunyan was origianally node.js
    // combined with CLI-tool cargo bunyan
    // turns spans into json format

    // EXPLAIN: std::io::stdout is just one example of sink
    // let formatting_layer =
    //     BunyanFormattingLayer::new(name, std::io::stdout);

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    // builder pattern:
    // we configure the behaviour of the subscriber
    // through changes or adding different layer
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
