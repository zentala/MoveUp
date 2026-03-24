---
id: E002-T02
epic: E002
status: done
original_id: T-OVR-002
---
# E002-T02: Device disconnected notification

Show a native Windows toast notification when the sensor is not connected. Emit `desk:device-missing` / `desk:device-lost` events from `serial.rs`. Throttled to max 1 notification per 5 minutes.
