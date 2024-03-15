# opentelemetry-system-metrics

This is my awesome crate enabling process level system metrics using opentelemetry.

Current metrics observed are:

- CPU
- Memory
- Disk
- Network
- GPU Memory

## Getting started

```bash
cargo add opentelemetry_system_metrics
```

```rust
use opentelemetry::global;
use opentelemetry_system_metrics::init_process_observer;

let meter = global::meter("process-meter");
init_process_observer(meter);
```

To get started with InfluxDB, you should create an account at InfluxDB Cloud, create a new telegraf opentelemetry exporter.

- Ex:

```bash
export INFLUX_TOKEN=<PROVIDED TOKEN>
telegraf --config <PROVIDED LINK>
cargo run --example otlp-tokio-metrics
```
