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
use eyre::ContextCompat;
use eyre::Result;
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::Nvml;

use sysinfo::PidExt;

use sysinfo::ProcessExt;
use sysinfo::SystemExt;
use sysinfo::{get_current_pid, System};

use opentelemetry::metrics::Meter;
use opentelemetry::KeyValue;

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
const DIRECTION: &str = "direction";

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
pub fn init_process_observer(meter: Meter) -> Result<()> {
    let pid =
        get_current_pid().map_err(|err| eyre::eyre!("could not get current pid. Error: {err}"))?;
    register_metrics(meter, pid)
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
/// init_process_observer_for_pid(meter, pid);
/// ```
///
pub fn init_process_observer_for_pid(meter: Meter, pid: u32) -> Result<()> {
    let pid = sysinfo::Pid::from_u32(pid);
    register_metrics(meter, pid)
}

fn register_metrics(meter: Meter, pid: sysinfo::Pid) -> Result<()> {
    let sys_ = System::new_all();
    let core_count = sys_
        .physical_core_count()
        .with_context(|| "Could not get physical core count")?;

    let nvml = Nvml::init();

    // CPU Usage
    meter
        .f64_observable_gauge(PROCESS_CPU_USAGE)
        .with_description("CPU usage of the process")
        .with_callback(move |observer| {
            let mut sys = System::new_all();
            sys.refresh_process(pid);

            if let Some(process) = sys.process(pid) {
                let cpu_usage = process.cpu_usage();
                observer.observe(cpu_usage.into(), &[]);
            }
        })
        .build();

    // CPU Utilization
    meter
        .f64_observable_gauge(PROCESS_CPU_UTILIZATION)
        .with_description("CPU utilization of the process as a fraction of core count")
        .with_callback(move |observer| {
            let mut sys = System::new_all();
            sys.refresh_process(pid);

            if let Some(process) = sys.process(pid) {
                let cpu_utilization = process.cpu_usage() / core_count as f32;
                observer.observe(cpu_utilization.into(), &[]);
            }
        })
        .build();

    // Memory Usage
    meter
        .i64_observable_gauge(PROCESS_MEMORY_USAGE)
        .with_description("Memory usage of the process")
        .with_callback(move |observer| {
            let mut sys = System::new_all();
            sys.refresh_process(pid);

            if let Some(process) = sys.process(pid) {
                observer.observe(process.memory() as i64, &[]);
            }
        })
        .build();

    // Virtual Memory Usage
    meter
        .i64_observable_gauge(PROCESS_MEMORY_VIRTUAL)
        .with_description("Virtual memory usage of the process")
        .with_callback(move |observer| {
            let mut sys = System::new_all();
            sys.refresh_process(pid);

            if let Some(process) = sys.process(pid) {
                observer.observe(process.virtual_memory() as i64, &[]);
            }
        })
        .build();

    // Disk I/O Read
    meter
        .i64_observable_gauge(PROCESS_DISK_IO)
        .with_description("Disk I/O read bytes of the process")
        .with_callback(move |observer| {
            let mut sys = System::new_all();
            sys.refresh_process(pid);

            if let Some(process) = sys.process(pid) {
                observer.observe(
                    process.disk_usage().read_bytes as i64,
                    &[KeyValue::new(DIRECTION, "read")],
                );
            }
        })
        .build();

    // Disk I/O Write
    meter
        .i64_observable_gauge(PROCESS_DISK_IO)
        .with_description("Disk I/O write bytes of the process")
        .with_callback(move |observer| {
            let mut sys = System::new_all();
            sys.refresh_process(pid);

            if let Some(process) = sys.process(pid) {
                observer.observe(
                    process.disk_usage().written_bytes as i64,
                    &[KeyValue::new(DIRECTION, "write")],
                );
            }
        })
        .build();

    // GPU Memory Usage
    meter
        .u64_observable_gauge(PROCESS_GPU_MEMORY_USAGE)
        .with_description("GPU memory usage of the process")
        .with_callback({
            move |observer| {
                if let Ok(nvml) = &nvml {
                    if let Ok(device) = nvml.device_by_index(0) {
                        if let Ok(gpu_stats) = device.running_compute_processes() {
                            for stat in gpu_stats {
                                if stat.pid == pid.as_u32() {
                                    let memory_used = match stat.used_gpu_memory {
                                        UsedGpuMemory::Used(bytes) => bytes,
                                        UsedGpuMemory::Unavailable => 0,
                                    };
                                    observer.observe(memory_used, &[]);
                                    return;
                                }
                            }
                        }
                    }
                }
                // Default to 0 if no matching GPU stats found
                observer.observe(0, &[]);
            }
        })
        .build();

    Ok(())
}
