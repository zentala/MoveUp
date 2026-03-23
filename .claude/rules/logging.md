# Logging & Debugging Reference

## Log location

All logs are in `{app_data_dir}/logs/` — typically:
```
C:\Users\{user}\AppData\Roaming\com.zentala.desk\logs\
```

## Minute snapshots

One JSON file per minute at `logs/YYYY-MM-DD/HH-MM.json`.

Contains full session state: state, height, sitting/standing seconds,
limits, score, connection status. ~500 bytes each, 1440/day max.

**Use case**: "What was the app state at 14:30?" → read `14-30.json`.

**Agent use**: read snapshots to verify data correctness, detect anomalies
(e.g. sitting_seconds jumping, standing_seconds not growing).

## Event log

Append-only text file at `logs/YYYY-MM-DD/events.log`.

One line per event, format: `HH:MM:SS TYPE details`.

Events logged:
- `STATE` — state transitions (Sitting→Standing, etc.)
- `CREDIT` — break credit applied (none/partial/full)
- `NOTIF` — notification fired (posture_balance, inactivity, etc.)
- `DEVICE` — sensor connected/lost
- `ALERT` — sit/stand limit alert
- `RESET` — daily counter reset
- `START` — app launch

**Use case**: "Why did I get a posture_balance notification?" →
grep `events.log` for `NOTIF posture_balance` — shows the ratio at fire time.

## Retention

Logs older than 7 days are auto-deleted on app startup.

## Debug tab

Settings → Debug tab shows live session data (polling every 1s).
See `DebugSection.tsx` for all displayed fields.

## Rust log levels

Set `RUST_LOG=desk_lib=debug` for verbose Rust logging to stderr.
This is separate from the file-based logging above.
