use opentelemetry::metrics::MeterProvider as _;
use opentelemetry_sdk::{metrics::SdkMeterProvider, Resource};
use opentelemetry_system_metrics::init_process_observer;
use std::time::Duration;
fn init_metrics() -> SdkMeterProvider {
    let exporter = opentelemetry_stdout::MetricExporterBuilder::default().build();
    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("metrics-basic-example")
                .build(),
        )
        .build()
}

#[tokio::main]
async fn main() {
    let meter_provider = init_metrics();
    let meter = meter_provider.meter("mylibraryname");
    let _ = init_process_observer(meter, Some(1)).await.unwrap();

    tokio::time::sleep(Duration::from_secs(60)).await;
    meter_provider.shutdown().unwrap();
}
