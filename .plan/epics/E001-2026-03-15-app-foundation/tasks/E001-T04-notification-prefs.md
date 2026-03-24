---
id: E001-T04
epic: E001
status: done
original_id: "0003"
---
# T003 — Notification Preference Toggles (3 types)

**Priority**: P2
**Status**: open
**Depends on**: T002

## Change from original design
`notify_position_changed` removed — would fire 20-40 toasts/day (spammy).
`notify_too_long_no_change` renamed to `notify_inactivity`.
`notify_too_long_one_position` renamed to `notify_daily_posture_balance`.

## The three notification types

| Config key | Trigger | Default |
|---|---|---|
| `notify_inactivity` | No position change for >= 90 min | true |
| `notify_daily_posture_balance` | Cumulative sitting today > 2x cumulative standing | true |
| `notify_praise_halfway` | Standing time today >= 50% of `stand_limit_mins x sessions` | true |

## Scope

### `session.rs`
- Add `last_position_change_at: Option<DateTime<Utc>>`
- In `apply_height()` on confirmed transition: update `last_position_change_at`
- Add `check_notification_conditions()` called from serial reader tick (every 60s):
  - `notify_inactivity`: `Utc::now() - last_position_change_at > 90min && config.notify_inactivity`
  - `notify_daily_posture_balance`: `sitting_secs > standing_secs * 2 && config.notify_daily_posture_balance`
  - Each fires at most once per hour (debounce flag per type, reset on position change)

### `notify_praise_halfway`
- Triggered in `apply_height()` when transitioning Sitting->Standing if `standing_secs >= (stand_limit_secs / 2)`
- Fires at most once per day (flag: `praise_halfway_fired_today`)

### Notification messages
- `notify_inactivity`: "No position change in 90 minutes — time to move."
- `notify_daily_posture_balance`: "You've been sitting most of today — consider standing for a while."
- `notify_praise_halfway`: "Halfway through your standing goal today — keep it up."

### Notification dispatch helper
```rust
fn fire_if_enabled(app: &AppHandle, config: &AppConfig, key: &str, message: &str) {
    // check config field by key, call tauri_plugin_notification
    // log::info!("notification fired: {}", key);
}
```

## Tests
- Unit: `notify_inactivity` not fired when toggle = false
- Unit: `notify_inactivity` fires after 90 min with no transition
- Unit: `notify_inactivity` does NOT fire twice within 60 min
- Unit: `notify_daily_posture_balance` fires when sitting_secs > standing_secs x 2
- Unit: `notify_praise_halfway` fires on Sitting->Standing when halfway reached
- Unit: `notify_praise_halfway` fires at most once per day

## Acceptance criteria
- [ ] Each toggle independently gates its notification
- [ ] No notification fires when `tauri-plugin-notification` not registered (graceful skip)
- [ ] All debounce flags reset correctly at midnight (T009 integration)
