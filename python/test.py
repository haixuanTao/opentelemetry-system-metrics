import otel_system_metrics

from time import sleep
import torch

_ = otel_system_metrics.init()


model = torch.hub.load("ultralytics/yolov5", "yolov5n")

print("Sleeping to wait for observability to send first data after interval...")
sleep(200)
