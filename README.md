# opentelemetry-system-metrics

This is my awesome crate Enabling system metrics from process to be observed using opentelemetry.
Current metrics observed are:
- CPU
- Memory
- Disk
- Network


## Getting started

```rust
use opentelemetry::global;
use opentelemetry_rust_system_metrics::init_process_observer;

let meter = global::meter("process-meter");
init_process_observer(meter);
```

