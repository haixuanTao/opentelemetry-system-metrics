use opentelemetry::metrics::MeterProvider as _;
use opentelemetry_sdk::{
    metrics::{MeterProvider, PeriodicReader},
    runtime,
};
use opentelemetry_system_metrics::init_process_observer;
use std::time::Duration;
fn init_metrics() -> MeterProvider {
    let exporter = opentelemetry_stdout::MetricsExporter::default();
    let reader = PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(Duration::from_secs(1))
        .build();
    MeterProvider::builder().with_reader(reader).build()
}

#[tokio::main]
async fn main() {
    let meter_provider = init_metrics();
    let meter = meter_provider.meter("mylibraryname");
    let _ = init_process_observer(meter).unwrap();

    tokio::time::sleep(Duration::from_secs(60)).await;
    meter_provider.shutdown().unwrap();
}
