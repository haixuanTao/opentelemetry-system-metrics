import otel_system_metrics
from time import sleep
import os

print("Initializing OpenTelemetry system metrics...")

# Initialize the metrics collector
runtime = otel_system_metrics.init()

print("System metrics initialized successfully!")
print(f"PID: {os.getpid()}")
print("Running for 60 seconds to collect and export metrics...")

# Run longer to ensure metrics are collected and exported
for i in range(12):  # 12 * 5 seconds = 60 seconds
    print(f"Running... {(i+1)*5} seconds elapsed")
    sleep(5)

print("Test completed - metrics should have been collected and sent to OTLP endpoint")