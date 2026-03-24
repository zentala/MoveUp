---
id: E002-T08
epic: E002
status: done
original_id: T-OVR-008
---
# E002-T08: DataSource refactor

Refactored `dev_mode` boolean to `DataSource { Demo, Live, Mock }` enum. Added `OVERLAY_DATA` env var. Demo = cycling animation (debug default), Live = real sensor data (release default), Mock = simulated 40-min sit + 10-min stand cycle.
