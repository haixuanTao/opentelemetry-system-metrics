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
//! use opentelemetry_rust_system_metrics::init_process_observer;
//!
//! let meter = global::meter("process-meter");
//! init_process_observer(meter);
//! ```
//!
use std::sync::Arc;
use std::sync::Mutex;
use sysinfo::PidExt;

use sysinfo::ProcessExt;
use sysinfo::SystemExt;
use sysinfo::{get_current_pid, System};

use opentelemetry::metrics::{BatchObserverResult, Meter};
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
const PROCESS_NETWORK_IO: &str = "process.network.io";
const DIRECTION: Key = Key::from_static_str("direction");

// Record asynchronnously information about the current process.
// # Example
//
// ```
// use opentelemetry::global;
// use opentelemetry_rust_system_metrics::init_process_observer;
//
// let meter = global::meter("process-meter");
// init_process_observer(meter);
// ```
//
pub fn init_process_observer(meter: Meter) {
    let sys = Arc::new(Mutex::new(System::new_all()));
    let mut sys_lock = sys.lock().unwrap();
    sys_lock.refresh_all();

    let pid = get_current_pid().unwrap();
    let core_count = sys_lock.physical_core_count().unwrap();

    meter
        .build_batch_observer(|batch| {
            let process_cpu_utilization = batch.f64_value_observer(PROCESS_CPU_USAGE).init();
            let process_cpu_usage = batch.f64_value_observer(PROCESS_CPU_UTILIZATION).init();
            let process_memory_usage = batch.i64_value_observer(PROCESS_MEMORY_USAGE).init();
            let process_memory_virtual = batch.u64_value_observer(PROCESS_MEMORY_VIRTUAL).init();
            let process_disk_io = batch.i64_value_observer(PROCESS_DISK_IO).init();
            let process_network_io = batch.u64_value_observer(PROCESS_NETWORK_IO).init();

            let sys = sys.clone();

            let common_attributes = if let Some(process) = sys_lock.process(pid) {
                [
                    PROCESS_PID.i64(pid.as_u32().into()),
                    PROCESS_EXECUTABLE_NAME.string(process.name().to_string()),
                    PROCESS_EXECUTABLE_PATH.string(process.exe().to_str().unwrap().to_string()),
                    PROCESS_COMMAND.string(process.cmd().concat().to_string()),
                ]
            } else {
                unimplemented!()
            };

            Ok(move |result: BatchObserverResult| {
                let mut sys_lock = sys.lock().unwrap();

                sys_lock.refresh_process(pid);

                if let Some(process) = sys_lock.process(pid) {
                    let cpu_usage = process.cpu_usage() / 100.;
                    let disk_io = process.disk_usage();
                    let network_io = process.network_usage();
                    result.observe(&[], &[process_cpu_usage.observation(cpu_usage.into())]);
                    result.observe(
                        &common_attributes,
                        &[process_cpu_utilization
                            .observation((cpu_usage / core_count as f32).into())],
                    );
                    result.observe(
                        &common_attributes,
                        &[process_memory_usage.observation(process.memory().try_into().unwrap())],
                    );
                    result.observe(
                        &common_attributes,
                        &[process_memory_virtual
                            .observation(process.virtual_memory().try_into().unwrap())],
                    );
                    result.observe(
                        &[common_attributes.as_slice(), &[DIRECTION.string("read")]].concat(),
                        &[process_disk_io.observation(disk_io.read_bytes.try_into().unwrap())],
                    );
                    result.observe(
                        &[common_attributes.as_slice(), &[DIRECTION.string("write")]].concat(),
                        &[process_disk_io.observation(disk_io.written_bytes.try_into().unwrap())],
                    );
                    result.observe(
                        &[common_attributes.as_slice(), &[DIRECTION.string("receive")]].concat(),
                        &[process_network_io
                            .observation(network_io.received_bytes.try_into().unwrap())],
                    );
                    result.observe(
                        &[
                            common_attributes.as_slice(),
                            &[DIRECTION.string("transmit")],
                        ]
                        .concat(),
                        &[process_network_io
                            .observation(network_io.transmitted_bytes.try_into().unwrap())],
                    );
                }
            })
        })
        .unwrap();
}
