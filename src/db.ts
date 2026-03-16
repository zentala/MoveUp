/**
 * db.ts — SQLite persistence layer for the Desk app.
 *
 * Initialises the database on first access by executing the schema DDL
 * retrieved from the Rust backend. Provides typed helpers for saving
 * sitting sessions and querying today's summary.
 */
import Database from "@tauri-apps/plugin-sql";
import { invoke } from "@tauri-apps/api/core";

let db: Database | null = null;

/**
 * Returns the singleton Database instance, initialising the schema on
 * first call. Safe to call multiple times — subsequent calls are no-ops.
 */
export async function getDb(): Promise<Database> {
  if (!db) {
    db = await Database.load("sqlite:desk.db");
    // Fetch schema DDL from Rust and execute each statement in order.
    const schema = await invoke<string>("get_schema_sql");
    for (const stmt of schema.split(";").map((s) => s.trim()).filter(Boolean)) {
      await db.execute(stmt, []);
    }
  }
  return db;
}

/**
 * Persists a completed sitting session to SQLite.
 *
 * @param startedAt   ISO-8601 string for when the sitting session started.
 * @param endedAt     ISO-8601 string for when it ended.
 * @param durationSecs Total duration in whole seconds.
 */
export async function saveSittingSession(
  startedAt: string,
  endedAt: string,
  durationSecs: number,
): Promise<void> {
  const d = await getDb();
  await d.execute(
    "INSERT INTO sessions (started_at, ended_at, state, duration_seconds) VALUES (?, ?, 'Sitting', ?)",
    [startedAt, endedAt, durationSecs],
  );
}

/** Row shape returned by the sessions SELECT query. */
interface SessionQueryRow {
  state: string;
  duration_seconds: number;
}

/**
 * Returns today's total sitting and standing seconds from SQLite.
 *
 * Only counts completed sessions (ended_at IS NOT NULL) that started today.
 */
export async function getTodaySummary(): Promise<{ sitting_secs: number; standing_secs: number }> {
  const d = await getDb();
  const today = new Date().toISOString().slice(0, 10);
  const rows = await d.select<SessionQueryRow[]>(
    "SELECT state, duration_seconds FROM sessions WHERE started_at LIKE ? AND ended_at IS NOT NULL",
    [`${today}%`],
  );
  const sitting_secs = rows
    .filter((r) => r.state === "Sitting")
    .reduce((s, r) => s + (r.duration_seconds ?? 0), 0);
  const standing_secs = rows
    .filter((r) => r.state !== "Sitting")
    .reduce((s, r) => s + (r.duration_seconds ?? 0), 0);
  return { sitting_secs, standing_secs };
}
