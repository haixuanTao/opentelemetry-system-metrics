use opentelemetry::metrics::MeterProvider as _;

use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::{MetricResult, SdkMeterProvider};
use opentelemetry_system_metrics::init_process_observer;
use std::time::Duration;

fn init_metrics() -> MetricResult<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .with_protocol(Protocol::Grpc)
        .with_timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(
        exporter,
        opentelemetry_sdk::runtime::Tokio,
    )
    .with_interval(std::time::Duration::from_secs(10))
    .with_timeout(Duration::from_secs(10))
    .build();

    Ok(opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .build())
}

#[tokio::main]
async fn main() {
    let meter_provider = init_metrics().unwrap();
    let meter = meter_provider.meter("mylibraryname");
    init_process_observer(meter).unwrap();

    // Do some work

    tokio::time::sleep(Duration::from_secs(120)).await
}
