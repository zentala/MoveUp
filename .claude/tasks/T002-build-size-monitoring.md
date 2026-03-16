# T002: Build Size Monitoring Script

**Task ID:** T002
**Status:** Pending
**Effort:** 4 hours
**Priority:** P0 (needs T001, blocks T005)
**Owner:** Subagent
**Date Created:** 2026-03-16

---

## Summary

Create `scripts/build-report.js` that reports installer size, tracks growth vs baseline, and logs to console + file. Add unit tests. Integrate with `pnpm tauri:build`.

---

## Acceptance Criteria

### 1. scripts/build-report.js (100-150 lines)

**Features:**

```javascript
// scripts/build-report.js
const fs = require('fs');
const path = require('path');

const BUNDLE_DIR = 'src-tauri/target/release/bundle';
const BASELINE_FILE = '.build-sizes.json';
const LOG_FILE = '.build-log.txt';

function run() {
  // 1. Platform check
  if (process.platform !== 'win32') {
    console.error('❌ build-report.js runs on Windows only');
    console.error(`   Current platform: ${process.platform}`);
    process.exit(1);
  }

  // 2. Scan for artifacts
  const nsis = scanNSIS();
  const msi = scanMSI();

  if (!nsis && !msi) {
    console.error('❌ No installers found in bundle/');
    console.error(`   Checked: ${BUNDLE_DIR}/nsis/ and ${BUNDLE_DIR}/msi/`);
    process.exit(1);
  }

  // 3. Load baseline
  let baseline = {};
  if (fs.existsSync(BASELINE_FILE)) {
    try {
      baseline = JSON.parse(fs.readFileSync(BASELINE_FILE, 'utf8'));
    } catch (e) {
      console.error(`❌ Corrupted baseline JSON: ${e.message}`);
      process.exit(1);
    }
  }

  // 4. Calculate metrics
  const metrics = {
    timestamp: new Date().toISOString(),
    nsis_bytes: nsis ? fs.statSync(nsis).size : null,
    msi_bytes: msi ? fs.statSync(msi).size : null,
  };

  // 5. Compare and alert
  const report = compareAndAlert(metrics, baseline);

  // 6. Log to console and file
  logReport(report, metrics);

  // 7. Update baseline (atomic write)
  writeAtomic(BASELINE_FILE, JSON.stringify(metrics, null, 2));

  process.exit(0);
}

function scanNSIS() {
  const dir = path.join(BUNDLE_DIR, 'nsis');
  if (!fs.existsSync(dir)) return null;
  const files = fs.readdirSync(dir).filter(f => f.endsWith('.exe'));
  return files.length ? path.join(dir, files[0]) : null;
}

function scanMSI() {
  const dir = path.join(BUNDLE_DIR, 'msi');
  if (!fs.existsSync(dir)) return null;
  const files = fs.readdirSync(dir).filter(f => f.endsWith('.msi'));
  return files.length ? path.join(dir, files[0]) : null;
}

function compareAndAlert(metrics, baseline) {
  const nsisGrowth = baseline.nsis_bytes
    ? ((metrics.nsis_bytes - baseline.nsis_bytes) / baseline.nsis_bytes * 100).toFixed(1)
    : null;
  const msiGrowth = baseline.msi_bytes
    ? ((metrics.msi_bytes - baseline.msi_bytes) / baseline.msi_bytes * 100).toFixed(1)
    : null;

  const alerts = [];
  if (metrics.nsis_bytes && metrics.nsis_bytes > 80e6) {
    alerts.push(`NSIS installer is ${(metrics.nsis_bytes / 1e6).toFixed(1)}MB (>80MB)`);
  }
  if (metrics.msi_bytes && metrics.msi_bytes > 80e6) {
    alerts.push(`MSI installer is ${(metrics.msi_bytes / 1e6).toFixed(1)}MB (>80MB)`);
  }
  if (nsisGrowth && nsisGrowth > 10) {
    alerts.push(`NSIS growth +${nsisGrowth}% vs baseline`);
  }
  if (msiGrowth && msiGrowth > 10) {
    alerts.push(`MSI growth +${msiGrowth}% vs baseline`);
  }

  return {
    nsis_bytes: metrics.nsis_bytes,
    msi_bytes: metrics.msi_bytes,
    nsis_growth: nsisGrowth,
    msi_growth: msiGrowth,
    alerts,
  };
}

function logReport(report, metrics) {
  const nsisLabel = report.nsis_bytes ? `${(report.nsis_bytes / 1e6).toFixed(1)} MB (nsis)` : 'N/A';
  const msiLabel = report.msi_bytes ? `${(report.msi_bytes / 1e6).toFixed(1)} MB (msi)` : 'N/A';

  const line1 = `✓ Build complete`;
  const line2 = `📦 Installer: ${nsisLabel}, ${msiLabel}`;
  const line3 = report.nsis_growth || report.msi_growth
    ? `📈 Growth: ${report.nsis_growth || report.msi_growth}%`
    : '';

  console.log(line1);
  console.log(line2);
  if (line3) console.log(line3);

  if (report.alerts.length > 0) {
    console.warn('⚠️  Warnings:');
    report.alerts.forEach(a => console.warn(`   - ${a}`));
  }

  // Log to file
  const logLine = `${metrics.timestamp} | ${nsisLabel} | ${msiLabel} | ${report.alerts.join('; ')}\n`;
  fs.appendFileSync(LOG_FILE, logLine);
}

function writeAtomic(file, content) {
  const temp = file + '.tmp';
  fs.writeFileSync(temp, content, 'utf8');
  fs.renameSync(temp, file); // atomic on all platforms
}

try {
  run();
} catch (e) {
  console.error(`❌ build-report.js error: ${e.message}`);
  process.exit(1);
}
```

**Decisions:**
- Exit code: 0 (success), 1 (fatal error)
- Platform check: Windows-only, helpful error message
- Hash: File size + mtime (no SHA256 for speed)
- Atomic write: Temp file → rename (prevents corruption on concurrent writes)

### 2. Package.json Integration

Update `apps/desk/package.json`:

```json
{
  "scripts": {
    "tauri:build": "pnpm test:all && tauri build && node scripts/build-report.js"
  }
}
```

### 3. Unit Tests: tests/scripts/build-report.test.js (~150 lines)

```javascript
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';

const TEMP_DIR = '.test-build-report';

beforeEach(() => {
  if (fs.existsSync(TEMP_DIR)) fs.rmSync(TEMP_DIR, { recursive: true });
  fs.mkdirSync(TEMP_DIR, { recursive: true });
});

afterEach(() => {
  if (fs.existsSync(TEMP_DIR)) fs.rmSync(TEMP_DIR, { recursive: true });
});

describe('build-report.js', () => {
  it('should scan NSIS and MSI artifacts', () => {
    // Mock artifacts
    const nsis = path.join(TEMP_DIR, 'nsis', 'zntlDesk_0.1.0_x64_en-US.exe');
    const msi = path.join(TEMP_DIR, 'msi', 'zntlDesk_0.1.0_x64_en-US.msi');
    fs.mkdirSync(path.dirname(nsis), { recursive: true });
    fs.mkdirSync(path.dirname(msi), { recursive: true });
    fs.writeFileSync(nsis, 'x'.repeat(65e6)); // 65 MB
    fs.writeFileSync(msi, 'x'.repeat(58e6));  // 58 MB

    // Run script (mock paths in tests)
    // Expect: sizes calculated, logged
  });

  it('should fail if no artifacts found', () => {
    // No artifacts created
    // Expect: exit code 1, error message
  });

  it('should detect corrupted baseline JSON', () => {
    fs.writeFileSync('.build-sizes.json', 'not valid json');
    // Expect: exit code 1, clear error
  });

  it('should alert if growth >10%', () => {
    // Create baseline: 60 MB
    // Create new: 66 MB
    // Expect: warning logged
  });

  it('should use atomic write', () => {
    // Verify temp file → rename pattern
  });
});
```

### 4. .build-sizes.json Format

Initial state (created by first run):

```json
{
  "timestamp": "2026-03-16T10:30:00.000Z",
  "nsis_bytes": 65000000,
  "msi_bytes": 58000000
}
```

Git-tracked (commit as baseline).

### 5. .build-log.txt Format

Git-ignored. Appended on each run:

```
2026-03-16T10:30:00.000Z | 65.0 MB (nsis), 58.0 MB (msi) |
2026-03-16T11:45:00.000Z | 65.2 MB (nsis), 58.1 MB (msi) | NSIS growth +0.3%
```

---

## Implementation Checklist

- [ ] Write `scripts/build-report.js` (100-150 lines)
- [ ] Add platform check (Windows-only)
- [ ] Implement artifact scanning (NSIS + MSI)
- [ ] Implement baseline comparison
- [ ] Implement atomic file write
- [ ] Update `package.json` tauri:build script
- [ ] Write `tests/scripts/build-report.test.js` (4-5 focused tests)
- [ ] Create `.build-sizes.json` (initial baseline, 0 bytes)
- [ ] Create `.build-log.txt` (empty, git-ignored)
- [ ] Test locally: `npm run tauri:build`
- [ ] Verify report output in console
- [ ] Verify `.build-sizes.json` updated
- [ ] Run unit tests: `npm test`

---

## Testing

```bash
# Local test
pnpm tauri:build
# Should see: "✓ Build complete" + sizes + growth (if baseline exists)

# Unit tests
pnpm test:scripts

# Concurrent runs (test atomic write safety)
pnpm tauri:build & pnpm tauri:build
# Both should succeed without baseline corruption
```

---

## Review References

- **Eng Review:** Architecture Issue #2 (run location: local + CI)
- **Eng Review:** Code Quality Issue #1 (error handling)
- **Eng Review:** Performance Issue #1 (hash strategy: skip SHA256)
- **CEO Review:** Issue #4 (race condition: atomic write)

---

## Blocks

- T005: Update CLAUDE.md (documents build-report.js)

## Depends On

- T001: Tauri config (artifacts must exist)
