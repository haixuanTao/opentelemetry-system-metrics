use opentelemetry::{
    global,
    metrics::{self},
};

use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::{metrics::MeterProvider, runtime};
use opentelemetry_system_metrics::init_process_observer;
use std::time::Duration;

fn init_metrics() -> metrics::Result<MeterProvider> {
    let export_config = ExportConfig {
        endpoint: "http://localhost:4317".to_string(),
        ..ExportConfig::default()
    };

    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(export_config),
        )
        .build()
}

#[tokio::main]
async fn main() {
    let _started = init_metrics();
    let meter = global::meter("process-meter");
    init_process_observer(meter);

    tokio::time::sleep(Duration::from_secs(120)).await
}
