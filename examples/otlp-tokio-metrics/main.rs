use opentelemetry::{global, InstrumentationScope};

use opentelemetry_otlp::MetricExporter;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_system_metrics::init_process_observer;
use std::time::Duration;

fn init_metrics() -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .build()
}

#[tokio::main]
async fn main() {
    let meter_provider = init_metrics();
    global::set_meter_provider(meter_provider.clone());
    let scope = InstrumentationScope::builder("basic")
        .with_version("1.0")
        .build();
    let meter = global::meter_with_scope(scope);

    println!("Start process observer");
    init_process_observer(meter).await.unwrap();
    println!("Finished process observer");

    // Do some work

    tokio::time::sleep(Duration::from_secs(120)).await
}
