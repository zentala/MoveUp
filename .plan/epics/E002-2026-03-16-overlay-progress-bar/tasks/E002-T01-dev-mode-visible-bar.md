---
id: E002-T01
epic: E002
status: done
original_id: T-OVR-001
---
# E002-T01: Dev mode — always-visible progress bar

Implement "dev mode" that makes the overlay bar visible without requiring a desk sensor. Auto-enabled in debug builds via `cfg!(debug_assertions)`. Cycles progress 0% -> 25% -> 50% -> 75% -> 100% every 25 seconds. Implemented in both OPAQUE and LAYERED render paths. 3 commits.
