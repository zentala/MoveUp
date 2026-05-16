/**
 * analyst-fixtures-builders.ts — Synthetic data builders for snapshots,
 * events, sessions, and daily KPIs.
 */
import type { BreakCredit, DeskState } from "@/types";
import {
  DAY_MS,
  HOUR_MS,
  dateLocal,
  seeded,
  type DailyKpi,
  type EventRow,
  type SessionRow,
  type SnapshotRow,
} from "./analyst-fixtures-types";

/** Build snapshot rows ~every 20 min over 7 days (~504 rows). */
export function buildSnapshots(today: Date = new Date()): SnapshotRow[] {
  const rng = seeded(0x5eed);
  const rows: SnapshotRow[] = [];
  const startMs = today.getTime() - 7 * DAY_MS;
  for (let day = 0; day < 7; day++) {
    let sitting = 0;
    let standing = 0;
    let score = 0;
    for (let slot = 0; slot < 72; slot++) {
      const t = startMs + day * DAY_MS + slot * 20 * 60 * 1000;
      const hour = new Date(t).getHours();
      let state: DeskState = "Away";
      let height = 72;
      const r = rng();
      if (hour >= 9 && hour < 18) {
        if (r < 0.62) {
          state = "Sitting";
          height = 72 + Math.round(rng() * 2);
          sitting += 1200;
        } else if (r < 0.88) {
          state = "Standing";
          height = 108 + Math.round(rng() * 6);
          standing += 1200;
          score += 1;
        } else if (r < 0.95) {
          state = "Walking";
          height = 110 + Math.round(rng() * 4);
        } else {
          state = "Away";
        }
      } else if (hour >= 7 && hour < 9) {
        state = rng() < 0.3 ? "Sitting" : "Away";
        height = state === "Sitting" ? 73 : 72;
      } else if (hour >= 18 && hour < 23) {
        state = rng() < 0.4 ? "Sitting" : "Away";
        height = state === "Sitting" ? 74 : 72;
        if (state === "Sitting") sitting += 1200;
      }
      rows.push({
        ts: new Date(t).toISOString(),
        state,
        deskHeightCm: height,
        sittingSecs: sitting,
        standingSecs: standing,
        breakSecs: standing,
        idleSecs: state === "Away" ? Math.round(rng() * 600) : 0,
        score,
      });
    }
  }
  return rows;
}

/** Build event log entries — ~200 over 7 days. */
export function buildEvents(today: Date = new Date()): EventRow[] {
  const rng = seeded(0xc0ffee);
  const rows: EventRow[] = [];
  const startMs = today.getTime() - 7 * DAY_MS;
  const kinds: EventRow["type"][] = [
    "STATE", "CREDIT", "NOTIF", "DEVICE", "ALERT", "RESET", "START", "AUTOSTART",
  ];
  for (let day = 0; day < 7; day++) {
    const dayStart = startMs + day * DAY_MS;
    rows.push({ ts: new Date(dayStart + 7 * HOUR_MS).toISOString(), type: "START", message: "app launched" });
    rows.push({ ts: new Date(dayStart + 7 * HOUR_MS + 1000).toISOString(), type: "AUTOSTART", message: "enabled" });
    rows.push({ ts: new Date(dayStart).toISOString(), type: "RESET", message: "daily counters reset" });
    const eventCount = 25 + Math.floor(rng() * 6);
    for (let i = 0; i < eventCount; i++) {
      const hour = 9 + Math.floor(rng() * 9);
      const min = Math.floor(rng() * 60);
      const t = dayStart + hour * HOUR_MS + min * 60_000;
      const k = kinds[Math.floor(rng() * kinds.length)];
      let msg = "";
      switch (k) {
        case "STATE": msg = "Sitting -> Standing"; break;
        case "CREDIT": msg = rng() < 0.5 ? "partial credit applied" : "full credit applied"; break;
        case "NOTIF": msg = "sit_limit step=2"; break;
        case "DEVICE": msg = rng() < 0.5 ? "connected COM3" : "lost"; break;
        case "ALERT": msg = "sit_limit reached"; break;
        case "RESET": msg = "daily score reset"; break;
        case "START": msg = "app launched"; break;
        case "AUTOSTART": msg = "service started"; break;
      }
      rows.push({ ts: new Date(t).toISOString(), type: k, message: msg });
    }
  }
  return rows.sort((a, b) => a.ts.localeCompare(b.ts));
}

/** Build session rows — 3-6 per day across 7 days. */
export function buildSessions(today: Date = new Date()): SessionRow[] {
  const rng = seeded(0xbeef);
  const rows: SessionRow[] = [];
  let id = 1;
  const startMs = today.getTime() - 7 * DAY_MS;
  for (let day = 0; day < 7; day++) {
    const dayStart = startMs + day * DAY_MS;
    const dateStr = dateLocal(today, 7 - day);
    const count = 3 + Math.floor(rng() * 4);
    let cursor = dayStart + 9 * HOUR_MS;
    for (let i = 0; i < count; i++) {
      const duration = 1500 + Math.floor(rng() * 1800);
      const standing = Math.floor(duration * (0.2 + rng() * 0.4));
      const sitting = duration - standing;
      const credit: BreakCredit = standing < 60
        ? "none"
        : standing < 120
          ? "partial"
          : "full";
      rows.push({
        id: id++,
        startedAt: new Date(cursor).toISOString(),
        endedAt: new Date(cursor + duration * 1000).toISOString(),
        state: "Sitting",
        durationSecs: duration,
        sittingSecs: sitting,
        standingSecs: standing,
        positionChanges: 1 + Math.floor(rng() * 4),
        breakCredit: credit,
        dateLocal: dateStr,
      });
      cursor += (duration + 600) * 1000;
    }
  }
  return rows;
}

/** Build daily KPI rollup — 7 days. */
export function buildDailyKpis(today: Date = new Date()): DailyKpi[] {
  const rng = seeded(0xf00d);
  const result: DailyKpi[] = [];
  for (let day = 6; day >= 0; day--) {
    const standingPct = 18 + Math.floor(rng() * 22);
    result.push({
      dateLocal: dateLocal(today, day),
      standingPct,
      positionChanges: 6 + Math.floor(rng() * 8),
      longestSessionSecs: 1800 + Math.floor(rng() * 1800),
      score: 35 + Math.floor(rng() * 50),
    });
  }
  return result;
}
