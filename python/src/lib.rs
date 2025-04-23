use opentelemetry::{global, InstrumentationScope};
use opentelemetry_otlp::MetricExporter;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_system_metrics::init_process_observer;
use pyo3::prelude::*;
use tokio::runtime::Builder;
use tokio::runtime::Runtime;

fn init_metrics() -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .build()
}

#[pyclass]
pub struct PyRuntime(Runtime);

/// Init the process metrics with optionally a given endpoint
#[pyfunction]
fn init() -> PyResult<PyRuntime> {
    let rt = Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    rt.spawn(async {
        let meter_provider = init_metrics();
        global::set_meter_provider(meter_provider.clone());
        let scope = InstrumentationScope::builder("basic")
            .with_version("1.0")
            .build();
        let meter = global::meter_with_scope(scope);
        init_process_observer(meter).await.unwrap();
    });
    Ok(PyRuntime(rt))
}

/// Process metrics module
#[pymodule]
fn otel_system_metrics(_py: Python, m: Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, &m)?)?;
    Ok(())
}
