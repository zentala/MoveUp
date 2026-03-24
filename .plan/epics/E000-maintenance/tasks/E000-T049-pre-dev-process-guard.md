---
id: E000-T049
epic: E000
status: done
created: 2026-03-23
completed: 2026-03-23
---
# E000-T049: Pre-dev process guard

## Problem
On Windows, `cargo` cannot replace `desk.exe` while a previous instance is running.
This causes `error: failed to remove file desk.exe (os error 5)` — a cryptic "Access Denied"
that doesn't explain the real cause. Developer loses time diagnosing it.

## Solution
A pre-flight script (`scripts/pre-dev.sh`) that runs before every `tauri dev` invocation.

### Behavior
1. Checks if `desk.exe` is already running via `tasklist`
2. If not running → exits silently, build proceeds
3. If running → shows clear message with PID and offers choice:
   - **[k] Kill** old process and start new build (default, auto-selects after 10s timeout)
   - **[s] Skip** — keep old process running, abort this build
4. `FORCE_KILL=1` env var or `--force` flag → auto-kills without prompt

### Usage
```bash
pnpm tauri:dev                  # demo (default)
pnpm tauri:dev:live             # real sensor
pnpm tauri:dev:mock             # simulated
pnpm tauri:dev -- --force       # auto-kill old instance
pnpm tauri:dev:live -- --force  # live + auto-kill
```

### Files
- `scripts/tauri-dev.sh` — unified launcher with process guard + flag parsing
- `package.json` — simplified scripts (all delegate to tauri-dev.sh)
