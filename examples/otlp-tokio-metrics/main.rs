use opentelemetry::{
    metrics::MeterProvider as _,
    metrics::{self},
};

use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::{metrics::SdkMeterProvider, runtime};
use opentelemetry_system_metrics::init_process_observer;
use std::time::Duration;

fn init_metrics() -> metrics::Result<SdkMeterProvider> {
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
        .with_period(Duration::from_secs(10))
        .build()
}

#[tokio::main]
async fn main() {
    let meter_provider = init_metrics().unwrap();
    let meter = meter_provider.meter("mylibraryname");
    init_process_observer(meter).unwrap();

    // Do some work

    tokio::time::sleep(Duration::from_secs(120)).await
}
