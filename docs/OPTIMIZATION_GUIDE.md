# Performance Optimization Guide

This guide documents current performance baselines, optimization strategies, and regression detection practices for zntlDesk.

## Current Baselines

### Memory Profile (as of 2026-03-16)

Measured via `pnpm test:perf` (10-second startup profile):

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Peak Memory | 229 MB | < 250 MB | ✓ Healthy |
| Stable Memory (after 5s) | 201 MB | < 200 MB | ⚠ Slightly over |
| Memory Trend | Growing | Stable | ⚠ Monitor |
| Sample Duration | 10 seconds | — | — |

**Interpretation:**
- App uses reasonable memory at startup
- Stable state around 200 MB is acceptable for Tauri + React + Rust backend
- "Growing" trend suggests memory is still being allocated during the 10-second window
- This is normal for startup; real concern is if it continues growing indefinitely

### Installer Size (baseline TBD)

- **Expected:** 60–70 MB (NSIS compressed)
- **Current:** Not yet measured (future build will populate `.build-sizes.json`)

## Optimization Strategies

### Bundle Size Optimization

**Goal:** Keep installer < 80 MB

#### 1. Tree-Shake Unused Code

```bash
# Check what's in the bundle
pnpm build --analyze
```

**Actions:**
- Remove unused Tauri plugins from `tauri.conf.json`
  - Only include: `notification`, `autostart`, `single-instance`, `store`, `sql`, `log`
  - Remove placeholder plugins (e.g., `shell`, `dialog` if not used)
- Remove unused React dependencies from `package.json`
- Use `/* webpackIgnore: true */` for large conditional imports

#### 2. React Code Splitting

**Current approach:**
- Main bundle: App + core components
- Lazy-loaded: Settings panel, calibration dialog

**Improvements:**
```typescript
const SettingsPanel = lazy(() => import('./components/SettingsPanel'));
const CalibrationDialog = lazy(() => import('./components/CalibrationDialog'));
```

**Benefits:**
- Users download main app first
- Heavy components load on-demand
- Typical savings: 10–20 KB per lazy component

#### 3. Disable Debug Symbols in Release

In `src-tauri/Cargo.toml`:

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization (slower build)
strip = true            # Strip debug symbols
```

**Typical savings:** 30–50% of binary size

#### 4. Minimize Tauri Plugin Count

Each plugin adds ~2–5 MB. Use only what's needed:

**Required:**
- `notification` — Break alerts
- `autostart` — Startup launch
- `single-instance` — Prevent duplicates
- `store` — User preferences
- `sql` — Database access
- `log` — Debugging

**Optional (remove if possible):**
- `shell` — Shell command execution (not used)
- `dialog` — File dialogs (not used)
- `fs` — File system (not used)

### Runtime Memory Optimization

**Goal:** Keep peak memory < 250 MB, stable < 200 MB

#### 1. Lazy-Load React Components

Load heavy components only when needed:

```typescript
// Bad: Imports on app startup
import SettingsPanel from './SettingsPanel';
import HistoryChart from './HistoryChart';

// Good: Import only when visible
const SettingsPanel = lazy(() => import('./SettingsPanel'));
const HistoryChart = lazy(() => import('./HistoryChart'));
```

**Typical savings:** 5–10 MB initial memory

#### 2. Batch Database Updates

Instead of single updates per reading:

```rust
// Bad: Update DB for every sensor reading (100ms intervals = 600 updates/min)
db.insert_reading(&reading)?;

// Good: Batch readings, update every 10 seconds
batch.push(reading);
if batch.len() >= 100 {
    db.insert_batch(&batch)?;
    batch.clear();
}
```

**Benefits:**
- Reduces database I/O
- Lower memory for transaction overhead
- Typical savings: 10–15 MB

#### 3. Archive Old Sessions

Periodically move old session data to archive table:

```rust
// On app startup, archive sessions older than 90 days
pub fn archive_old_sessions(db: &Database) -> Result<()> {
    let cutoff = now() - Duration::days(90);
    db.execute(
        "INSERT INTO sessions_archive SELECT * FROM sessions WHERE date < ?",
        [cutoff],
    )?;
    db.execute("DELETE FROM sessions WHERE date < ?", [cutoff])?;
    Ok(())
}
```

**Timeline:** Monthly cleanup via background task

**Typical savings:** 20–30 MB after 1 year of data

#### 4. Memory Pooling (Future)

Pre-allocate buffers for common operations:

```rust
// Pre-allocate 100 session objects on startup
pub struct SessionPool {
    available: Vec<Session>,
    in_use: Vec<Session>,
}

impl SessionPool {
    pub fn get(&mut self) -> Session {
        self.available.pop().unwrap_or_default()
    }

    pub fn return_session(&mut self, session: Session) {
        self.available.push(session);
    }
}
```

**Benefit:** Reduces heap fragmentation, improves GC
**Effort:** Medium complexity
**Typical savings:** 5–10 MB

### Database Optimization

#### 1. VACUUM on Startup

After archiving or deleting data, reclaim disk space:

```rust
pub fn startup_maintenance(db: &Database) -> Result<()> {
    db.execute("VACUUM", [])?;  // Rebuild database file
    db.execute("ANALYZE", [])?; // Update statistics
    Ok(())
}
```

**Frequency:** On app startup (one-time)
**Typical savings:** 20–50% of database file size

#### 2. Indexes on Hot Columns

Ensure frequently-queried columns are indexed:

```sql
CREATE INDEX idx_sessions_date ON sessions(date);
CREATE INDEX idx_readings_timestamp ON readings(timestamp);
CREATE INDEX idx_readings_session_id ON readings(session_id);
```

**Query examples:**
- "Show all sessions from today" → uses `idx_sessions_date`
- "Get readings for session X" → uses `idx_readings_session_id`

**Benefit:** Faster queries = less memory holding result sets

#### 3. Partition by Date (Future)

For large datasets (> 1 year):

```sql
-- Create partition for 2026
CREATE TABLE sessions_2026 (
    -- same schema as sessions
);

-- Create trigger to route inserts
CREATE TRIGGER sessions_partition
BEFORE INSERT ON sessions
BEGIN
  INSERT INTO sessions_2026 VALUES (NEW.*);
END;
```

**Benefit:** Smaller active table, better cache locality

## Regression Detection Checklist

Use this checklist before every commit or merge:

### For Dependencies

- [ ] New npm package added?
  - [ ] Run `npm info <package>` — check size (typical: 20–200 KB)
  - [ ] Verify not duplicated (`pnpm ls <package>`)
  - [ ] If > 500 KB, consider alternatives
  - [ ] Benchmark before/after: `pnpm build --analyze`

- [ ] New Rust crate added?
  - [ ] Check `Cargo.lock` — new dependencies?
  - [ ] Run `cargo build --release` — check binary size
  - [ ] If binary grows > 2 MB, review the crate

### Before Merge

- [ ] Tests pass: `pnpm test:all`
- [ ] Check bundle size growth:
  ```bash
  pnpm build:report
  ```
  - Look at `.build-sizes.json`
  - Compare timestamp vs previous baseline
  - If growth > 5 MB, investigate

- [ ] No new memory leaks:
  - [ ] Disable cache: open DevTools (F12)
  - [ ] Hard-reload: Ctrl+Shift+R
  - [ ] Open Task Manager → check memory after 30s
  - [ ] If jumping > 50 MB during reload, investigate

### Before Release

**Run full performance profile:**

```bash
# 1. Kill any running instances
taskkill /IM zntlDesk.exe /F 2>/dev/null || true

# 2. Run performance test
pnpm test:perf

# 3. Check baseline
cat .perf-baseline.json
```

**Acceptance:**
- [ ] Peak memory < 300 MB (alert: > 300)
- [ ] Stable memory < 220 MB (target: < 200)
- [ ] Growth vs previous baseline < 20 MB
- [ ] No memory leaks over 1-minute idle period

**If fails:**
- [ ] Review recent commits for allocations
- [ ] Use Valgrind (Linux) or Windows Task Manager profiler
- [ ] Profile hot functions in Rust (use `perf`, `flamegraph`)
- [ ] Defer release until fixed

## Performance Monitoring

### How to Monitor Over Time

1. **After each release**, commit updated `.perf-baseline.json` and `.build-sizes.json`
2. **Track in Git history:**
   ```bash
   git log -p .perf-baseline.json | grep peak_mb
   ```
3. **Set CI/CD gates:**
   - Build fails if memory > 300 MB
   - Build warns if growth > 20 MB

### Tools

- **Memory profiling:** `pnpm test:perf` (via Windows Task Manager)
- **Bundle analysis:** `pnpm build --analyze` (Vite plugin)
- **CPU profiling:** Firefox DevTools Performance tab (React)
- **Rust profiling:** `cargo flamegraph` (Linux/WSL)

## Common Issues and Fixes

### Issue: Memory grows indefinitely

**Symptoms:**
- App starts at 180 MB
- After 5 minutes: 220 MB
- After 30 minutes: 350 MB+

**Likely causes:**
- Event listeners not unsubscribed
- React component not cleanup in `useEffect`
- Rust thread accumulating allocations

**Fix:**
```typescript
// React cleanup
useEffect(() => {
  const unsubscribe = eventBus.on('desk:reading', handler);
  return () => unsubscribe();  // ← Clean up on unmount
}, []);

// Rust cleanup
impl Drop for SessionManager {
    fn drop(&mut self) {
        self.shutdown();  // ← Cleanup threads
    }
}
```

### Issue: Bundle size jumped 10 MB

**Likely causes:**
- New dependency with heavy binary
- Debug symbols not stripped
- Unused code not tree-shaken

**Fix:**
1. Check what changed:
   ```bash
   git diff HEAD~1 package.json src-tauri/Cargo.toml
   ```
2. Remove unnecessary dependencies
3. Ensure release build is configured correctly

### Issue: App feels slow/sluggish

**Symptoms:**
- UI lag when opening settings
- Tray icon updates are delayed
- Database queries are slow

**Diagnosis:**
1. Open DevTools (F12) → Performance tab
2. Record a session (click recording while doing action)
3. Look for long tasks (red bars > 100ms)
4. Check React render times in DevTools → Profiler

**Common fixes:**
- Memoize expensive components: `React.memo(Component)`
- Batch state updates: `flushSync()` sparingly
- Index database queries (see above)

## Summary

| Strategy | Effort | Savings | Frequency |
|----------|--------|---------|-----------|
| Tree-shake code | Low | 5–10 MB | Per release |
| Lazy-load components | Low | 5–10 MB | Ongoing |
| Strip debug symbols | Low | 30–50% | Release profile |
| Batch DB updates | Medium | 10–15 MB | Architecture |
| Archive old data | Medium | 20–30 MB | Monthly |
| Memory pooling | High | 5–10 MB | Future |
| Database partitioning | High | Varies | Year+ horizon |

**Priority:** Focus on tree-shaking, lazy-loading, and debug symbol stripping first (low effort, good savings).

## References

- [CLAUDE.md - Memory Profiling](../CLAUDE.md#memory-profiling-on-demand)
- [CLAUDE.md - Pre-Release Checklist](../CLAUDE.md#pre-release-checklist)
- [Tauri Bundle Optimization](https://tauri.app/en/develop/guides/bundle-your-application/#optimizing-the-binary)
- [Rust Release Profile](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [React Performance Profiler](https://react.dev/reference/react/Profiler)
