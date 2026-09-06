/**
 * format.ts — utility functions for formatting time and display values.
 */

import type { DeskState } from "@/types";

/** Maps desk state to ProgressBar color scheme. */
export function colorSchemeFor(state: DeskState): "sitting" | "standing" | "gray" {
  if (state === "Sitting") return "sitting";
  if (state === "Standing") return "standing";
  return "gray";
}

/**
 * Formats a duration in seconds to `hh:mm:ss` string.
 * @param totalSeconds - Non-negative integer number of seconds.
 * @returns Formatted string like "1:04:32" or "0:32:07".
 */
export function formatDuration(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = s % 60;

  const mm = String(minutes).padStart(2, "0");
  const ss = String(seconds).padStart(2, "0");

  if (hours > 0) {
    return `${hours}:${mm}:${ss}`;
  }
  return `${mm}:${ss}`;
}

/**
 * Formats a duration in seconds as a human-readable short form: "2h14m" or "45m" or "32s".
 * @param totalSeconds - Non-negative integer number of seconds.
 */
export function formatDurationShort(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = s % 60;

  if (hours > 0) {
    return `${hours}h${String(minutes).padStart(2, "0")}m`;
  }
  if (minutes > 0) {
    return `${minutes}m`;
  }
  return `${seconds}s`;
}

/** Two-digit zero-padded number, e.g. `7` → `"07"`. */
function pad2(n: number): string {
  return String(n).padStart(2, "0");
}

const MONTHS = [
  "Jan", "Feb", "Mar", "Apr", "May", "Jun",
  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/**
 * Renders a `YYYY-MM-DD` day as `06 Sep 2026`.
 * @param date - Calendar day in `YYYY-MM-DD` form.
 */
export function formatDate(date: string): string {
  const d = new Date(`${date}T00:00:00`);
  return `${date.slice(8, 10)} ${d.toLocaleDateString("en-US", {
    month: "short",
  })} ${date.slice(0, 4)}`;
}

/**
 * Renders a `YYYY-MM-DD` day as its full weekday name, e.g. `Sunday`.
 * @param date - Calendar day in `YYYY-MM-DD` form.
 */
export function weekdayName(date: string): string {
  const d = new Date(`${date}T00:00:00`);
  return d.toLocaleDateString("en-US", { weekday: "long" });
}

/**
 * Renders an epoch-millis timestamp as local `HH:MM:SS`.
 * @returns `null` when the input is `null`, `undefined` or `0`.
 */
export function formatRefreshedAt(epoch: number | null): string | null {
  if (!epoch) return null;
  const d = new Date(epoch);
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}:${pad2(d.getSeconds())}`;
}

/** Renders a `YYYY-MM-DD` pair as `May 10–17` or `May 28–Jun 3`. */
export function formatRangeLabel(from: string, to: string): string {
  const fParts = from.split("-").map(Number);
  const tParts = to.split("-").map(Number);
  if (fParts.length !== 3 || tParts.length !== 3) return `${from} → ${to}`;
  const fm = MONTHS[fParts[1] - 1] ?? "?";
  const tm = MONTHS[tParts[1] - 1] ?? "?";
  if (fm === tm) return `${fm} ${fParts[2]}–${tParts[2]}`;
  return `${fm} ${fParts[2]}–${tm} ${tParts[2]}`;
}

/** Formats idle seconds as `"Xm Ys"`. */
export function formatIdleTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}m ${s}s`;
}

/** Renders an ISO timestamp as local `HH:MM`. */
export function formatTime(iso: string): string {
  const d = new Date(iso);
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
}
