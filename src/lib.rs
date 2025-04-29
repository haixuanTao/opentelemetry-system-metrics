//! This is my awesome crate Enabling system metrics from process to be observed using opentelemetry.
//! Current metrics observed are:
//! - CPU
//! - Memory
//! - Disk
//! - Network
//!
//!
//! # Getting started
//!
//! To use this crate, add the following to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! opentelemetry = "0.29"
//! opentelemetry-system-metrics = "0.4"
//! tokio = { version = "1", features = ["full"] }
//! sysinfo = "0.34"
//! nvml-wrapper = "0.10"
//! eyre = { version = "0.6", features = ["tokio"] }
//! tracing = "0.1"
//! ```
//!
//! ```
//! use opentelemetry::global;
//! use opentelemetry_system_metrics::init_process_observer;
//!
//! #[tokio::main]
//! async fn main() {
//!     use opentelemetry_system_metrics::init_process_observer_once;
//!     let meter = global::meter("process-meter");
//!     let result = init_process_observer_once(meter).await;
//! }
//! ```
//!

use eyre::ContextCompat;
use eyre::Result;
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::Nvml;
use opentelemetry::metrics::Meter;
use opentelemetry::Key;
use opentelemetry::KeyValue;
use std::time::Duration;
use sysinfo::{get_current_pid, System};
use tokio::time::sleep;
use tracing::warn;

const PROCESS_PID: Key = Key::from_static_str("process.pid");
const PROCESS_EXECUTABLE_NAME: Key = Key::from_static_str("process.executable.name");
const PROCESS_EXECUTABLE_PATH: Key = Key::from_static_str("process.executable.path");
const PROCESS_COMMAND: Key = Key::from_static_str("process.command");
const PROCESS_CPU_USAGE: &str = "process.cpu.usage";
const PROCESS_CPU_UTILIZATION: &str = "process.cpu.utilization";
const PROCESS_MEMORY_USAGE: &str = "process.memory.usage";
const PROCESS_MEMORY_VIRTUAL: &str = "process.memory.virtual";
const PROCESS_DISK_IO: &str = "process.disk.io";
// const PROCESS_NETWORK_IO: &str = "process.network.io";
const DIRECTION: Key = Key::from_static_str("direction");
const PROCESS_GPU_MEMORY_USAGE: &str = "process.gpu.memory.usage";

/// Record asynchronously information about the current process.
///
/// # Parameters
/// * `meter`: The OpenTelemetry meter to use for recording metrics.
pub async fn init_process_observer(meter: Meter) -> Result<()> {
    let pid =
        get_current_pid().map_err(|err| eyre::eyre!("could not get current pid. Error: {err}"))?;
    register_metrics(meter, pid, None).await
}

/// Record asynchronously information about a specific process by its PID.
///
/// # Parameters
/// * `meter`: The OpenTelemetry meter to use for recording metrics.
/// * `pid`: The PID of the process to observe.
pub async fn init_process_observer_for_pid(meter: Meter, pid: u32) -> Result<()> {
    let pid = sysinfo::Pid::from_u32(pid);
    register_metrics(meter, pid, None).await
}

/// Record asynchronously information about the current process once.
///
/// # Parameters
/// * `meter`: The OpenTelemetry meter to use for recording metrics.
///
/// # Example
/// ```
/// use opentelemetry::global;
/// use opentelemetry_system_metrics::init_process_observer_once;
///
/// #[tokio::main]
/// async fn main() {
///    let meter = global::meter("process-meter");
///   let result = init_process_observer_once(meter).await;
/// }
/// ```
pub async fn init_process_observer_once(meter: Meter) -> Result<()> {
    let pid =
        get_current_pid().map_err(|err| eyre::eyre!("could not get current pid. Error: {err}"))?;
    register_metrics(meter, pid, Some(1)).await
}

/// Register metrics for the current process.
///
/// # Parameters
/// * `meter`: The OpenTelemetry meter to use for recording metrics.
/// * `pid`: The PID of the process to observe.
/// * `iterations`: Optional number of iterations to run the observer. If `None`, it will run indefinitely.
///
async fn register_metrics(
    meter: Meter,
    pid: sysinfo::Pid,
    iterations: Option<usize>,
) -> Result<()> {
    let core_count =
        System::physical_core_count().with_context(|| "Could not get physical core count")?;

    let nvml = Nvml::init();

    let process_cpu_utilization = meter
        .f64_gauge(PROCESS_CPU_USAGE)
        .with_description("The percentage of CPU in use.")
        .with_unit("percent")
        .build();
    let process_cpu_usage = meter
        .f64_gauge(PROCESS_CPU_UTILIZATION)
        .with_description("The amount of CPU in use.")
        .with_unit("percent")
        .build();
    let process_memory_usage = meter
        .i64_gauge(PROCESS_MEMORY_USAGE)
        .with_description("The amount of physical memory in use.")
        .with_unit("byte")
        .build();
    let process_memory_virtual = meter
        .i64_gauge(PROCESS_MEMORY_VIRTUAL)
        .with_description("The amount of committed virtual memory.")
        .with_unit("byte")
        .build();
    let process_disk_io = meter
        .i64_gauge(PROCESS_DISK_IO)
        .with_description("Disk bytes transferred.")
        .with_unit("byte")
        .build();

    let process_gpu_memory_usage = meter
        .u64_gauge(PROCESS_GPU_MEMORY_USAGE)
        .with_description("The amount of physical GPU memory in use.")
        .with_unit("byte")
        .build();

    let mut sys = System::new_all();
    sys.refresh_all();

    let common_attributes = if let Some(process) = sys.process(pid) {
        [
            KeyValue::new(PROCESS_PID, pid.as_u32().clone() as i64),
            KeyValue::new(
                PROCESS_EXECUTABLE_NAME,
                process
                    .name()
                    .to_os_string()
                    .into_string()
                    .unwrap_or_default(),
            ),
            KeyValue::new(
                PROCESS_EXECUTABLE_PATH,
                process
                    .exe()
                    .map(|path| path.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ),
            KeyValue::new(
                PROCESS_COMMAND,
                process.cmd().iter().fold(String::new(), |t1, t2| {
                    t1 + " " + t2.to_str().unwrap_or_default()
                }),
            ),
        ]
    } else {
        unimplemented!()
    };

    let interval = std::env::var("OTEL_METRIC_EXPORT_INTERVAL")
        .unwrap_or_else(|_| "30000".to_string())
        .parse::<u64>()
        .unwrap_or(30000);

    let mut counter = 0;
    loop {
        sleep(Duration::from_millis(interval)).await;
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);

        if let Some(process) = sys.process(pid) {
            let cpu_usage = process.cpu_usage();
            let disk_io = process.disk_usage();
            // let status = process.status();

            process_cpu_usage.record(cpu_usage.into(), &[]);
            process_cpu_utilization
                .record((cpu_usage / core_count as f32).into(), &common_attributes);
            process_memory_usage.record((process.memory()).try_into()?, &common_attributes);
            process_memory_virtual
                .record((process.virtual_memory()).try_into()?, &common_attributes);
            process_disk_io.record(
                disk_io.read_bytes.try_into()?,
                &[
                    common_attributes.as_slice(),
                    &[KeyValue::new(DIRECTION, "read")],
                ]
                .concat(),
            );
            process_disk_io.record(
                disk_io.written_bytes.try_into()?,
                &[
                    common_attributes.as_slice(),
                    &[KeyValue::new(DIRECTION, "write")],
                ]
                .concat(),
            );
            if let Some(max) = iterations {
                counter += 1;
                if counter >= max && max > 0 {
                    break Ok(());
                }
            }
        }

        // let mut last_timestamp = last_timestamp.lock().unwrap().clone();
        match &nvml {
            Ok(nvml) => {
                // Get the first `Device` (GPU) in the system
                if let Ok(device) = nvml.device_by_index(0) {
                    if let Ok(gpu_stats) = device.running_compute_processes() {
                        for stat in gpu_stats.iter() {
                            if stat.pid == pid.as_u32() {
                                let memory_used = match stat.used_gpu_memory {
                                    UsedGpuMemory::Used(bytes) => bytes,
                                    UsedGpuMemory::Unavailable => 0,
                                };

                                process_gpu_memory_usage.record(memory_used, &common_attributes);

                                break;
                            }
                        }
                    };
                }
            }
            Err(_) => {
                // If we can't get the NVML, we just put 0.
                warn!("Could not get NVML, recording 0 for GPU memory usage");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::global;
    use tokio::runtime::Runtime;

    #[test]
    fn test_init_process_observer_once() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let meter = global::meter("test-meter");
            let result = init_process_observer_once(meter).await;
            assert!(result.is_ok());
        });
    }
}
