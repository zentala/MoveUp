# T002 — Config Store: Persist Settings + Calibration (Rust Owns Truth)

**Priority**: P0
**Status**: open
**Absorbs**: T008 (calibration persistence)

## Architectural decision
**Rust is the single source of truth** for today's session stats.
`SessionManager` seeds from SQLite on startup. Frontend reads via Tauri commands only.
`getTodaySummary()` in TypeScript (plugin-sql) is **removed** — replaced by `get_today_summary` Tauri command.

## Goal
1. Persist all user config + calibration across restarts via `tauri-plugin-store`
2. Replace hardcoded `DEFAULT_SESSION_LIMIT_SECS = 2400` (40 min → 45 min, step=5)
3. On startup: load config → seed `SessionManager` with stored calibration + limits

## Scope

### New `src-tauri/src/config.rs`
```rust
/// All user-configurable settings. Every field has #[serde(default)] so
/// a corrupt or missing store entry falls back to defaults cleanly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_sit_limit")]
    pub sit_limit_mins: u32,          // default 45, step 5, range 10–90
    #[serde(default = "default_stand_limit")]
    pub stand_limit_mins: u32,        // default 15, step 5, range 5–60
    #[serde(default = "bool_true")]
    pub notify_inactivity: bool,      // too long without any position change
    #[serde(default = "bool_true")]
    pub notify_daily_posture_balance: bool,  // cumulative time skewed today
    #[serde(default = "bool_true")]
    pub notify_praise_halfway: bool,  // praise for 50% of standing goal
    // Calibration (formerly only in SessionManager, now persisted):
    #[serde(default = "default_sitting_mm")]
    pub sitting_mm: i32,              // default 720
    #[serde(default = "default_standing_mm")]
    pub standing_mm: i32,             // default 1050
    #[serde(default = "default_thickness_mm")]
    pub desk_thickness_mm: i32,       // default 30
}

impl AppConfig {
    /// Load from store, or return defaults if missing/corrupt.
    pub fn load(store: &Store) -> Self { ... }
    /// Clamp all values to valid ranges after deserialize.
    pub fn clamped(mut self) -> Self {
        self.sit_limit_mins   = self.sit_limit_mins.clamp(10, 90);
        self.stand_limit_mins = self.stand_limit_mins.clamp(5, 60);
        self.sitting_mm       = self.sitting_mm.clamp(400, 900);
        self.standing_mm      = self.standing_mm.clamp(900, 1400);
        // Validate non-inverted calibration:
        if self.sitting_mm >= self.standing_mm {
            self.sitting_mm  = 720;
            self.standing_mm = 1050;
            log::warn!("calibration inverted — reset to defaults");
        }
        self
    }
}
```

### `session.rs` init change
```rust
impl SessionManager {
    pub fn new_from_config(config: &AppConfig) -> Self {
        Self {
            // seed limits from config
            state: SessionState {
                session_limit_secs: config.sit_limit_mins as i64 * 60,
                ...
            },
            sitting_height_cm: config.sitting_mm as f32 / 10.0,
            standing_height_cm: config.standing_mm as f32 / 10.0,
            desk_thickness_cm: config.desk_thickness_mm as f32 / 10.0,
            stand_limit_secs: config.stand_limit_mins as i64 * 60,
            ...
        }
    }
    /// Called on startup: seed today's totals from SQLite so in-memory
    /// counters survive restarts.
    pub fn load_today_totals(&mut self, sitting_secs: i64, standing_secs: i64) {
        self.state.sitting_seconds  = sitting_secs;
        self.state.standing_seconds = standing_secs;
        log::info!("seeded today totals: sitting={}s standing={}s", sitting_secs, standing_secs);
    }
}
```

### `commands.rs` — new commands (move to `config.rs` module)
- `get_settings() -> AppConfig`
- `save_settings(config: AppConfig)` → validates via `config.clamped()`, writes store, updates `SessionManager`
- `get_today_summary() -> TodaySummary` — **real SQLite query** (replaces placeholder)

### `db.ts` (TypeScript) — **remove** `getTodaySummary()` and `saveSittingSession()`
These are replaced by Rust-side DB operations.

## Renamed notification fields
- `notify_too_long_no_change` → `notify_inactivity`
- `notify_too_long_one_position` → `notify_daily_posture_balance`
- `notify_position_changed` → **removed**

## DEFAULT_SESSION_LIMIT_SECS change
Update `session.rs`: `DEFAULT_SESSION_LIMIT_SECS = 2700` (45 min).
Update all references to 2400.

## Tests
- Unit: `AppConfig::clamped()` rejects inverted calibration, logs warning
- Unit: `AppConfig::clamped()` clamps sit_limit_mins = 999 → 90
- Unit: `AppConfig` deserializes with missing fields → all defaults
- Unit: `SessionManager::new_from_config()` applies calibration mm → cm correctly
- Integration: `save_settings` → restart → `get_settings` returns saved values

## Acceptance criteria
- [ ] Calibration survives restart
- [ ] Sit/stand limits survive restart
- [ ] Corrupt store falls back to defaults (no crash)
- [ ] Inverted calibration auto-corrects with log warning
- [ ] `get_today_summary` returns real SQLite data (no more `standing_secs: 0`)
- [ ] Frontend `getTodaySummary()` (JS) removed
