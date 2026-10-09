# Lifecycle review follow-ups

Source: independent T11 review of 26d4618. Blocking DB-contention readiness and
prior-failure classification are handled in the task's review rounds. Independent
loopback readiness is accepted into T05. These additional observations are not
silently included in the current correction round.

| Observation | Importance | Points | Done |
|---|---|---|---|
| Early diagnostic files use a separate UUID in their filename and have no cleanup policy or EventLogger path link; make correlation and retention explicit. | Low | 2 | No |
| Preserve safe error classification when reporting Tauri build and DB initialization errors; current fixed codes lose prior error detail. | Low | 1 | No |
| The global panic-hook recursion guard suppresses a concurrent panic's prior hook; assess a per-thread guard and concurrent-panic fixture. | Low | 1 | No |
| Document the intended failed readiness after any observed panic, including caught task panics, before connecting recovery. | Low | ? | No |
| Add unconditional child cleanup to application_lifecycle.rs so failed assertions cannot leave its synthetic holder alive. | Low | 1 | No |
| Rust formatting failed because the component is unavailable; review formatting and public API comments without silently installing a global toolchain. | Low | 2 | No |
| Update PROJECT.xml for new modules during accepted integration; this repository requirement was outside the worker's write set. | Low | 1 | No |
| Consider a clearer lifecycle state for intentional shutdown instead of initializing with quit_requested; preserve the ready=false contract. | Low | 1 | No |

Unconfirmed pre-existing observation: no explicit managed Store<Wry> registration
was found in application source. Verify the plugin's actual registration behavior
before claiming the existing session store flush is broken. Importance Medium,
Points 3, Done No; not a confirmed regression from this task.
