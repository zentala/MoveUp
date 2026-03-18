/**
 * Overlay Screenshot Testing
 *
 * Captures screenshots of the top bar every 500ms while app is running
 * Synchronizes with logs to correlate visual changes with frame numbers
 */

import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';

test.describe('Overlay Bar Visual Testing', () => {
    let screenshotDir: string;
    let testStartTime: number;
    let frameCounter = 0;

    test.beforeAll(() => {
        // Create timestamped screenshot directory
        const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
        screenshotDir = path.join('.claude/test-runs', timestamp, 'screenshots');

        // Create directory if it doesn't exist
        if (!fs.existsSync(screenshotDir)) {
            fs.mkdirSync(screenshotDir, { recursive: true });
        }

        testStartTime = Date.now();
        console.log(`\n📸 Screenshot test started`);
        console.log(`📁 Saving to: ${screenshotDir}`);
    });

    test('capture top bar screenshots every 500ms for 30 seconds', async ({ page }) => {
        // Navigate to app window
        await page.goto('http://localhost:1443', { waitUntil: 'networkidle' });

        // Capture viewport dimensions
        const viewport = page.viewportSize();
        if (!viewport) {
            throw new Error('No viewport size available');
        }

        const captureInterval = 500; // ms
        const testDuration = 30000; // 30 seconds
        const topBarHeight = 10; // pixels to capture from top
        const screenshots: { time: number; frame: number; path: string }[] = [];

        console.log(`\n📹 Capturing ${topBarHeight}px from top, every ${captureInterval}ms`);
        console.log(`⏱️ Duration: ${testDuration / 1000} seconds`);
        console.log(`📸 Expected frames: ~${Math.ceil(testDuration / captureInterval)}`);
        console.log('');

        const startTime = Date.now();

        while (Date.now() - startTime < testDuration) {
            const elapsedMs = Date.now() - startTime;
            const elapsedSec = (elapsedMs / 1000).toFixed(1);

            // Capture screenshot of full viewport
            const screenshotPath = path.join(screenshotDir, `frame-${frameCounter}.png`);
            await page.screenshot({ path: screenshotPath });

            // Log with timestamp
            const timeLabel = `[${elapsedSec}s]`;
            console.log(`${timeLabel} Frame ${frameCounter}: ${screenshotPath}`);

            screenshots.push({
                time: elapsedMs,
                frame: frameCounter,
                path: screenshotPath
            });

            frameCounter++;

            // Wait for next capture
            await page.waitForTimeout(captureInterval);
        }

        // Write manifest of all screenshots
        const manifestPath = path.join(screenshotDir, 'manifest.json');
        fs.writeFileSync(manifestPath, JSON.stringify({
            testStartTime: new Date(testStartTime).toISOString(),
            testDuration,
            captureInterval,
            totalFrames: frameCounter,
            screenshots
        }, null, 2));

        console.log(`\n✅ Captured ${frameCounter} screenshots`);
        console.log(`📋 Manifest: ${manifestPath}`);

        // Generate analysis
        generateScreenshotAnalysis(screenshotDir, frameCounter);
    });

    test.afterAll(async () => {
        console.log(`\n📊 Test complete. Check screenshots in: ${screenshotDir}`);
    });
});

/**
 * Generate analysis document comparing screenshots over time
 */
function generateScreenshotAnalysis(screenshotDir: string, frameCount: number) {
    const analysisPath = path.join(screenshotDir, 'ANALYSIS.md');

    let analysis = `# Overlay Bar Screenshot Analysis\n\n`;
    analysis += `**Total Frames:** ${frameCount}\n`;
    analysis += `**Duration:** ${frameCount * 500}ms (${(frameCount * 500 / 1000).toFixed(1)}s)\n`;
    analysis += `**Capture Interval:** 500ms\n\n`;

    analysis += `## Visual Expectations (DEMO Mode: 0% → 25% → 50% → 75% → 100% every 5 seconds)\n\n`;
    analysis += `| Time | Expected Stage | Expected Progress | Expected Bar Width |\n`;
    analysis += `|------|---|---|---|\n`;

    const stages = [
        { time: '0-5s', stage: 0, progress: '0%', width: '~0px (min 1px)' },
        { time: '5-10s', stage: 1, progress: '25%', width: '~480px (1920/4)' },
        { time: '10-15s', stage: 2, progress: '50%', width: '~960px (1920/2)' },
        { time: '15-20s', stage: 3, progress: '75%', width: '~1440px (1920*3/4)' },
        { time: '20-25s', stage: 4, progress: '100%', width: '1920px (full)' },
        { time: '25-30s', stage: 0, progress: '0%', width: '~0px (min 1px) [cycle restarts]' }
    ];

    stages.forEach(s => {
        analysis += `| ${s.time} | ${s.stage} | ${s.progress} | ${s.width} |\n`;
    });

    analysis += `\n## How to Verify\n\n`;
    analysis += `1. Open frame-0.png, frame-10.png (5s mark), frame-20.png (10s), etc.\n`;
    analysis += `2. Look at top of screenshot for white bar at top-left\n`;
    analysis += `3. Check if bar grows wider over time as expected\n`;
    analysis += `4. Note any flickering or color changes\n`;
    analysis += `5. Look for blinking pattern (should be smooth, not flickering)\n\n`;

    analysis += `## Questions to Answer\n\n`;
    analysis += `- [ ] Is white bar visible in frame-0.png?\n`;
    analysis += `- [ ] Does bar grow wider at frame-10 (~5s)?\n`;
    analysis += `- [ ] Is it noticeably wider at frame-20 (~10s)?\n`;
    analysis += `- [ ] Full width at frame-40 (~20s)?\n`;
    analysis += `- [ ] Any blinking/flickering visible?\n`;
    analysis += `- [ ] Does bar width correlate with time (0% → 25% → 50% → 75% → 100%)?\n`;
    analysis += `- [ ] Consistent white color (no color cycling)?\n\n`;

    analysis += `## Frame Numbering Reference\n\n`;
    analysis += `- Frame 0 = 0.0s\n`;
    analysis += `- Frame 10 = 5.0s (expect stage 1: 25%)\n`;
    analysis += `- Frame 20 = 10.0s (expect stage 2: 50%)\n`;
    analysis += `- Frame 30 = 15.0s (expect stage 3: 75%)\n`;
    analysis += `- Frame 40 = 20.0s (expect stage 4: 100%)\n`;
    analysis += `- Frame 50 = 25.0s (expect stage 0 again: 0%)\n`;
    analysis += `- Frame 60 = 30.0s (expect stage 1 again: 25%)\n\n`;

    analysis += `## File Locations\n\n`;
    analysis += `- Screenshots: frame-0.png through frame-${frameCount - 1}.png\n`;
    analysis += `- Manifest: manifest.json\n`;
    analysis += `- Logs: ../overlay.log (correlate with screenshots)\n`;

    fs.writeFileSync(analysisPath, analysis);
    console.log(`\n📊 Analysis template: ${analysisPath}`);
}
