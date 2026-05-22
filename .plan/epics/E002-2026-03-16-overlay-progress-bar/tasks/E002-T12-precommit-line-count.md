---
id: E002-T12
epic: E002
status: completed
original_id: T-OVR-012
title: Precommit hook for file line count
---
# E002-T12: Precommit hook for file line count

Added precommit hook that checks all staged `.rs`, `.ts`, `.tsx` files. Fails if any exceed 250 lines. Excludes test files and generated code. Prevents files from growing beyond the limit again.
