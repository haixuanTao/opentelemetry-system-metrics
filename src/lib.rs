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
//! ```
//! use opentelemetry::global;
//! use opentelemetry_system_metrics::init_process_observer;
//!
//! let meter = global::meter("process-meter");
//! init_process_observer(meter);
//! ```
//!

use std::time::Duration;

use eyre::ContextCompat;
use eyre::Result;
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::Nvml;

use opentelemetry::KeyValue;
use sysinfo::{get_current_pid, System};

use opentelemetry::metrics::Meter;
use opentelemetry::Key;
use tokio::time::sleep;

const PROCESS_PID: Key = Key::from_static_str("process.pid");
const PROCESS_EXECUTABLE_NAME: Key = Key::from_static_str("process.executable.name");
const PROCESS_EXECUTABLE_PATH: Key = Key::from_static_str("process.executable.path");
const PROCESS_COMMAND: Key = Key::from_static_str("process.command");

// Not implemented yet!
//
// const PROCESS_COMMAND_LINE: Key = Key::from_static_str("process.command_line");
// const PROCESS_COMMAND_ARGS: Key = Key::from_static_str("process.command_args");
// const PROCESS_OWNER: Key = Key::from_static_str("process.owner");

const PROCESS_CPU_USAGE: &str = "process.cpu.usage";
const PROCESS_CPU_UTILIZATION: &str = "process.cpu.utilization";
const PROCESS_MEMORY_USAGE: &str = "process.memory.usage";
const PROCESS_MEMORY_VIRTUAL: &str = "process.memory.virtual";
const PROCESS_DISK_IO: &str = "process.disk.io";
// const PROCESS_NETWORK_IO: &str = "process.network.io";
const DIRECTION: Key = Key::from_static_str("direction");

// const PROCESS_GPU_USAGE: &str = "process.gpu.usage";
const PROCESS_GPU_MEMORY_USAGE: &str = "process.gpu.memory.usage";

/// Record asynchronnously information about the current process.
/// # Example
///
/// ```
/// use opentelemetry::global;
/// use opentelemetry_system_metrics::init_process_observer;
///
/// let meter = global::meter("process-meter");
/// init_process_observer(meter);
/// ```
///
pub async fn init_process_observer(meter: Meter) -> Result<()> {
    let pid =
        get_current_pid().map_err(|err| eyre::eyre!("could not get current pid. Error: {err}"))?;
    register_metrics(meter, pid).await
}

/// Record asynchronously information about a specific process by its PID.
/// # Example
///
/// ```
/// use opentelemetry::global;
/// use opentelemetry_system_metrics::init_process_observer_for_pid;
///
/// let meter = global::meter("process-meter");
/// let pid = 1234; // replace with the actual PID
/// init_process_observer_for_pid(meter, pid).await;
/// ```
///
pub async fn init_process_observer_for_pid(meter: Meter, pid: u32) -> Result<()> {
    let pid = sysinfo::Pid::from_u32(pid);
    register_metrics(meter, pid).await
}

async fn register_metrics(meter: Meter, pid: sysinfo::Pid) -> Result<()> {
    let sys_ = System::new_all();
    let core_count = sys_
        .physical_core_count()
        .with_context(|| "Could not get physical core count")?;

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

    loop {
        sleep(Duration::from_millis(500)).await;
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);

        if let Some(process) = sys.process(pid) {
            let cpu_usage = process.cpu_usage();
            let disk_io = process.disk_usage();
            // let status = process.status();

            process_cpu_usage.record(cpu_usage.into(), &[]);
            process_cpu_utilization
                .record((cpu_usage / core_count as f32).into(), &common_attributes);
            process_memory_usage.record((process.memory()).try_into().unwrap(), &common_attributes);
            process_memory_virtual.record(
                (process.virtual_memory()).try_into().unwrap(),
                &common_attributes,
            );
            process_disk_io.record(
                disk_io.read_bytes.try_into().unwrap(),
                &[
                    common_attributes.as_slice(),
                    &[KeyValue::new(DIRECTION, "read")],
                ]
                .concat(),
            );
            process_disk_io.record(
                disk_io.written_bytes.try_into().unwrap(),
                &[
                    common_attributes.as_slice(),
                    &[KeyValue::new(DIRECTION, "write")],
                ]
                .concat(),
            );
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

                        // If the loop finishes and no pid matched our pid, put 0.
                        process_gpu_memory_usage.record(0, &common_attributes);
                    };
                }
            }
            Err(_) => {
                // If we can't get the NVML, we just put 0.
                process_gpu_memory_usage.record(0, &common_attributes);
            }
        }
    }
}
