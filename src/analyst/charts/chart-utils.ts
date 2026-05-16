/**
 * chart-utils.ts — Shared SVG chart helpers for the Analyst dashboard.
 */

export interface Margin {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export const DEFAULT_MARGIN: Margin = { top: 12, right: 16, bottom: 28, left: 40 };

/** Linear scale from [d0, d1] domain to [r0, r1] range. */
export function scaleLinear(d0: number, d1: number, r0: number, r1: number) {
  const dd = d1 - d0 || 1;
  return (v: number) => r0 + ((v - d0) / dd) * (r1 - r0);
}

/** Build N evenly spaced ticks across [d0, d1]. */
export function ticks(d0: number, d1: number, n: number): number[] {
  if (n <= 1) return [d0, d1];
  const step = (d1 - d0) / (n - 1);
  return Array.from({ length: n }, (_, i) => d0 + i * step);
}

/** Chart color palette — keep aligned with desk semantic colors. */
export const chartColors = {
  axis: "#555",
  axisText: "#999",
  gridline: "#2a2a3a",
  primary: "#DAA520",
  sitting: "#8B0000",
  standing: "#3CB371",
  walking: "#4682B4",
  away: "#666",
  score: "#DAA520",
  bar: "#5a8fbf",
  warn: "#ffc107",
  alert: "#f44336",
  background: "#1a1a2e",
  card: "#0d0d1a",
  text: "#ddd",
  subtext: "#888",
};

/** Round to 2 decimals. */
export function r2(n: number): number {
  return Math.round(n * 100) / 100;
}
