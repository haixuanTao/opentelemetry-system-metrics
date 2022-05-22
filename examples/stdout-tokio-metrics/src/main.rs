use futures::stream::Stream;
use futures::StreamExt;
use opentelemetry::global;
use opentelemetry::sdk::export::metrics::stdout;
use opentelemetry::sdk::metrics::PushController;
use opentelemetry_system_metrics::init_process_observer;
use std::time::Duration;

// Skip first immediate tick from tokio, not needed for async_std.
fn delayed_interval(duration: Duration) -> impl Stream<Item = tokio::time::Instant> {
    opentelemetry::sdk::util::tokio_interval_stream(duration).skip(1)
}

pub fn init_meter() -> PushController {
    stdout(tokio::spawn, delayed_interval).init()
}

#[tokio::main]
async fn main() {
    let _started = init_meter();
    let meter = global::meter("process-meter");
    init_process_observer(meter);

    tokio::time::sleep(Duration::from_secs(60)).await
}
