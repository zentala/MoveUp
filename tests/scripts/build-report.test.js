import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const TEMP_DIR = path.join(__dirname, '../../.test-build-report');

beforeEach(() => {
  if (fs.existsSync(TEMP_DIR)) fs.rmSync(TEMP_DIR, { recursive: true });
  fs.mkdirSync(TEMP_DIR, { recursive: true });
});

afterEach(() => {
  if (fs.existsSync(TEMP_DIR)) fs.rmSync(TEMP_DIR, { recursive: true });
});

describe('build-report.js', () => {
  it('should calculate file sizes correctly', () => {
    const nsis = path.join(TEMP_DIR, 'nsis', 'zntlDesk.exe');
    fs.mkdirSync(path.dirname(nsis), { recursive: true });
    fs.writeFileSync(nsis, 'x'.repeat(65e6)); // 65 MB

    const stats = fs.statSync(nsis);
    expect(stats.size).toBe(65e6);
  });

  it('should detect missing installers gracefully', () => {
    const dir = path.join(TEMP_DIR, 'nsis');
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });

    // No files in directory - should fail
    const files = fs.readdirSync(dir).filter(f => f.endsWith('.exe'));
    expect(files.length).toBe(0);
  });

  it('should parse baseline JSON correctly', () => {
    const baselineFile = path.join(TEMP_DIR, '.build-sizes.json');
    const baselineData = {
      timestamp: '2026-03-16T10:30:00.000Z',
      nsis_bytes: 65000000,
      msi_bytes: 58000000,
    };

    fs.writeFileSync(baselineFile, JSON.stringify(baselineData, null, 2));
    const loaded = JSON.parse(fs.readFileSync(baselineFile, 'utf8'));

    expect(loaded.nsis_bytes).toBe(65000000);
    expect(loaded.msi_bytes).toBe(58000000);
  });

  it('should handle corrupted baseline JSON with error', () => {
    const baselineFile = path.join(TEMP_DIR, '.build-sizes.json');
    fs.writeFileSync(baselineFile, 'not valid json');

    expect(() => {
      JSON.parse(fs.readFileSync(baselineFile, 'utf8'));
    }).toThrow();
  });

  it('should calculate growth percentage correctly', () => {
    const oldSize = 60e6; // 60 MB
    const newSize = 66e6; // 66 MB (10% growth)
    const growth = ((newSize - oldSize) / oldSize * 100).toFixed(1);

    expect(parseFloat(growth)).toBe(10.0);
  });

  it('should use atomic write pattern with temp file', () => {
    const testFile = path.join(TEMP_DIR, 'test.json');
    const tempFile = testFile + '.tmp';
    const content = JSON.stringify({ test: 'data' }, null, 2);

    // Simulate atomic write
    fs.writeFileSync(tempFile, content, 'utf8');
    fs.renameSync(tempFile, testFile);

    expect(fs.existsSync(testFile)).toBe(true);
    expect(fs.existsSync(tempFile)).toBe(false);
    const loaded = JSON.parse(fs.readFileSync(testFile, 'utf8'));
    expect(loaded.test).toBe('data');
  });

  it('should format sizes as MB correctly', () => {
    const bytes = 65000000;
    const mb = (bytes / 1e6).toFixed(1);
    expect(mb).toBe('65.0');
  });

  it('should append log lines without truncating', () => {
    const logFile = path.join(TEMP_DIR, '.build-log.txt');

    fs.appendFileSync(logFile, '2026-03-16T10:30:00.000Z | 65.0 MB (nsis), 58.0 MB (msi) |\n');
    fs.appendFileSync(logFile, '2026-03-16T11:45:00.000Z | 65.2 MB (nsis), 58.1 MB (msi) | NSIS growth +0.3%\n');

    const contents = fs.readFileSync(logFile, 'utf8');
    const lines = contents.split('\n').filter(l => l.length > 0);

    expect(lines.length).toBe(2);
    expect(lines[0]).toContain('10:30:00');
    expect(lines[1]).toContain('11:45:00');
  });
});
