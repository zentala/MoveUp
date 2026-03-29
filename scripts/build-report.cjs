const fs = require('fs');
const path = require('path');

// In a pnpm monorepo, Tauri builds to the workspace root target/ dir
const BUNDLE_DIR_MONOREPO = '../../target/release/bundle';
const BUNDLE_DIR_LOCAL = 'src-tauri/target/release/bundle';
const BUNDLE_DIR = fs.existsSync(BUNDLE_DIR_MONOREPO) ? BUNDLE_DIR_MONOREPO : BUNDLE_DIR_LOCAL;
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
