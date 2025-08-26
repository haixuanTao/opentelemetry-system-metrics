import otel_system_metrics
from time import sleep
import os

print("Initializing OpenTelemetry system metrics...")

# Initialize the metrics collector
runtime = otel_system_metrics.init()

print("System metrics initialized successfully!")
print(f"PID: {os.getpid()}")
print("Sleeping for 10 seconds to allow metrics collection...")

# Sleep for a shorter time to see if metrics are being collected
sleep(10)

print("Test completed - metrics should have been collected and sent to OTLP endpoint")