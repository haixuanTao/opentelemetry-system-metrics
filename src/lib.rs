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
use opentelemetry::metrics::Unit;

use sysinfo::PidExt;

use sysinfo::ProcessExt;
use sysinfo::SystemExt;
use sysinfo::{get_current_pid, System};

use opentelemetry::metrics::Meter;
use opentelemetry::Key;

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

// Record asynchronnously information about the current process.
// # Example
//
// ```
// use opentelemetry::global;
// use opentelemetry_system_metrics::init_process_observer;
//
// let meter = global::meter("process-meter");
// init_process_observer(meter);
// ```
//
pub fn init_process_observer(meter: Meter) {
    let pid = get_current_pid().unwrap();
    let sys_ = System::new_all();
    let core_count = sys_.physical_core_count().unwrap();
    let process_cpu_utilization = meter
        .f64_observable_gauge(PROCESS_CPU_USAGE)
        .with_description("The percentage of CPU in use.")
        .init();
    let process_cpu_usage = meter.f64_observable_gauge(PROCESS_CPU_UTILIZATION).init();
    let process_memory_usage = meter
        .i64_observable_gauge(PROCESS_MEMORY_USAGE)
        .with_description("The amount of physical memory in use.")
        .with_unit(Unit::new("byte"))
        .init();
    let process_memory_virtual = meter
        .i64_observable_gauge(PROCESS_MEMORY_VIRTUAL)
        .with_description("The amount of committed virtual memory.")
        .with_unit(Unit::new("byte"))
        .init();
    let process_disk_io = meter
        .i64_observable_gauge(PROCESS_DISK_IO)
        .with_description("Disk bytes transferred.")
        .with_unit(Unit::new("byte"))
        .init();

    meter
        .register_callback(
            &[
                process_cpu_utilization.as_any(),
                process_cpu_usage.as_any(),
                process_memory_usage.as_any(),
                process_memory_virtual.as_any(),
                process_disk_io.as_any(),
            ],
            move |context| {
                let mut sys = System::new_all();
                sys.refresh_processes();

                let common_attributes = if let Some(process) = sys.process(pid) {
                    [
                        PROCESS_PID.i64(pid.as_u32().into()),
                        PROCESS_EXECUTABLE_NAME.string(process.name().to_string()),
                        PROCESS_EXECUTABLE_PATH.string(process.exe().to_str().unwrap().to_string()),
                        PROCESS_COMMAND.string(process.cmd().join(" ").to_string()),
                    ]
                } else {
                    unimplemented!()
                };

                sys.refresh_process(pid);

                if let Some(process) = sys.process(pid) {
                    let cpu_usage = process.cpu_usage() / 100.;
                    let disk_io = process.disk_usage();
                    // let network_io = process.network_usage();

                    context.observe_f64(&process_cpu_usage, cpu_usage.into(), &[]);
                    context.observe_f64(
                        &process_cpu_utilization,
                        (cpu_usage / core_count as f32).into(),
                        &common_attributes,
                    );
                    context.observe_i64(
                        &process_memory_usage,
                        (process.memory()).try_into().unwrap(),
                        &common_attributes,
                    );
                    context.observe_i64(
                        &process_memory_virtual,
                        (process.virtual_memory()).try_into().unwrap(),
                        &common_attributes,
                    );
                    context.observe_i64(
                        &process_disk_io,
                        disk_io.read_bytes.try_into().unwrap(),
                        &[common_attributes.as_slice(), &[DIRECTION.string("read")]].concat(),
                    );
                    context.observe_i64(
                        &process_disk_io,
                        disk_io.written_bytes.try_into().unwrap(),
                        &[common_attributes.as_slice(), &[DIRECTION.string("write")]].concat(),
                    );

                    // result.observe(
                    //     &[common_attributes.as_slice(), &[DIRECTION.string("receive")]].concat(),
                    //     &[process_network_io
                    //         .observe(context,(network_io.received_bytes.try_into().unwrap())],
                    // );
                    // result.observe(
                    //     &[
                    //         common_attributes.as_slice(),
                    //         &[DIRECTION.string("transmit")],
                    //     ]
                    //     .concat(),
                    //     &[process_network_io
                    //         .observe(context,(network_io.transmitted_bytes.try_into().unwrap())],
                    // );
                }
            },
        )
        .unwrap();
}
