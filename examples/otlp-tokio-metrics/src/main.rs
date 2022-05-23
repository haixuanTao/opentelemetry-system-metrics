use futures::stream::Stream;
use futures::StreamExt;
use opentelemetry::sdk::metrics::selectors;
use opentelemetry::sdk::metrics::PushController;
use opentelemetry::{global, metrics};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_system_metrics::init_process_observer;
use std::time::Duration;

// Skip first immediate tick from tokio, not needed for async_std.
fn delayed_interval(duration: Duration) -> impl Stream<Item = tokio::time::Instant> {
    opentelemetry::sdk::util::tokio_interval_stream(duration).skip(1)
}

fn init_meter() -> metrics::Result<PushController> {
    let export_config = ExportConfig::default();
    opentelemetry_otlp::new_pipeline()
        .metrics(tokio::spawn, delayed_interval)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .with_aggregator_selector(selectors::simple::Selector::Exact)
        .build()
}

#[tokio::main]
async fn main() {
    let _started = init_meter();
    let meter = global::meter("process-meter");
    init_process_observer(meter);

    tokio::time::sleep(Duration::from_secs(120)).await
}
