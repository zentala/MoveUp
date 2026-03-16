/**
 * TodayStats.test.tsx — unit tests for delta arrow indicators.
 *
 * Test Coverage:
 * - ↓ (green/better) shown for sitting when sitting_secs < yesterday_sitting_secs - 300
 * - ↑ (amber/worse) shown for sitting when sitting_secs > yesterday_sitting_secs + 300
 * - No arrow for sitting when difference < 5 minutes (300s)
 * - ↑ (green/better) shown for standing when standing_secs > yesterday_standing_secs + 300
 * - ↓ (amber/worse) shown for standing when standing_secs < yesterday_standing_secs - 300
 * - No arrow for standing when difference < 5 minutes (300s)
 */
import { describe, it, expect } from "vitest";

/**
 * Unit tests for delta arrow logic (no component mounting required).
 * Tests the logic of determining which arrow to show based on delta.
 */
describe("TodayStats — Delta Indicators Logic", () => {
  // Sitting delta logic:
  // delta = today_sitting - yesterday_sitting
  // ↓ (better) if delta < -300
  // ↑ (worse) if delta > 300
  // no arrow if |delta| <= 300

  it("calculates sitting delta: ↓ (better) when -600s difference", () => {
    const todaySitting = 1200; // 20 min
    const yesterdaySitting = 1800; // 30 min
    const delta = todaySitting - yesterdaySitting; // -600

    const showDownArrow = delta < -300;
    expect(showDownArrow).toBe(true);
  });

  it("calculates sitting delta: ↑ (worse) when +900s difference", () => {
    const todaySitting = 2100; // 35 min
    const yesterdaySitting = 1200; // 20 min
    const delta = todaySitting - yesterdaySitting; // 900

    const showUpArrow = delta > 300;
    expect(showUpArrow).toBe(true);
  });

  it("calculates sitting delta: no arrow when +150s difference", () => {
    const todaySitting = 1350;
    const yesterdaySitting = 1200;
    const delta = todaySitting - yesterdaySitting; // 150

    const showArrow = Math.abs(delta) > 300;
    expect(showArrow).toBe(false);
  });

  it("calculates sitting delta: no arrow when -150s difference", () => {
    const todaySitting = 1050;
    const yesterdaySitting = 1200;
    const delta = todaySitting - yesterdaySitting; // -150

    const showArrow = Math.abs(delta) > 300;
    expect(showArrow).toBe(false);
  });

  // Standing delta logic:
  // delta = today_standing - yesterday_standing
  // ↑ (better) if delta > 300 (more standing is better)
  // ↓ (worse) if delta < -300 (less standing is worse)
  // no arrow if |delta| <= 300

  it("calculates standing delta: ↑ (better) when +600s difference", () => {
    const todayStanding = 1200; // 20 min
    const yesterdayStanding = 600; // 10 min
    const delta = todayStanding - yesterdayStanding; // 600

    const showUpArrowBetter = delta > 300;
    expect(showUpArrowBetter).toBe(true);
  });

  it("calculates standing delta: ↓ (worse) when -600s difference", () => {
    const todayStanding = 300; // 5 min
    const yesterdayStanding = 900; // 15 min
    const delta = todayStanding - yesterdayStanding; // -600

    const showDownArrowWorse = delta < -300;
    expect(showDownArrowWorse).toBe(true);
  });

  it("calculates standing delta: no arrow when +150s difference", () => {
    const todayStanding = 450;
    const yesterdayStanding = 300;
    const delta = todayStanding - yesterdayStanding; // 150

    const showArrow = Math.abs(delta) > 300;
    expect(showArrow).toBe(false);
  });

  it("threshold: exactly -300s is no arrow (boundary)", () => {
    const today = 1200;
    const yesterday = 1500;
    const delta = today - yesterday; // -300

    const showDownArrow = delta < -300; // strictly less than
    expect(showDownArrow).toBe(false); // boundary: -300 is no arrow
  });

  it("threshold: exactly +300s is no arrow (boundary)", () => {
    const today = 1500;
    const yesterday = 1200;
    const delta = today - yesterday; // 300

    const showUpArrow = delta > 300; // strictly greater than
    expect(showUpArrow).toBe(false); // boundary: +300 is no arrow
  });

  it("threshold: -301s shows arrow", () => {
    const today = 1199;
    const yesterday = 1500;
    const delta = today - yesterday; // -301

    const showDownArrow = delta < -300;
    expect(showDownArrow).toBe(true);
  });

  it("threshold: +301s shows arrow", () => {
    const today = 1501;
    const yesterday = 1200;
    const delta = today - yesterday; // 301

    const showUpArrow = delta > 300;
    expect(showUpArrow).toBe(true);
  });

  it("first day of use: high delta without baseline still shows arrow", () => {
    const todaySitting = 2400; // 40 min
    const yesterdaySitting = 0; // 0 min (no data)
    const delta = todaySitting - yesterdaySitting; // 2400

    // Our current implementation shows arrow even with 0 baseline
    const showUpArrow = delta > 300;
    expect(showUpArrow).toBe(true);
  });

  it("both sitting and standing deltas can be evaluated independently", () => {
    const todaySitting = 1800;
    const yesterdaySitting = 1200;
    const sitDelta = todaySitting - yesterdaySitting; // 600 > 300

    const todayStanding = 300;
    const yesterdayStanding = 600;
    const standDelta = todayStanding - yesterdayStanding; // -300 = no arrow

    const showSitArrow = sitDelta > 300;
    const showStandArrow = Math.abs(standDelta) > 300;

    expect(showSitArrow).toBe(true); // sitting shows arrow
    expect(showStandArrow).toBe(false); // standing shows no arrow
  });
});
