/**
 * useTimer.ts — hook that increments a seconds counter at 1 Hz.
 *
 * Used to drive a live session timer display without depending on event
 * frequency from the Rust backend.
 */
import { useEffect, useState } from "react";

/**
 * Returns a seconds counter that increments every second starting from
 * `initialSeconds`. Resets when `initialSeconds` changes.
 *
 * @param initialSeconds - Starting value for the counter.
 * @param running - When false the counter pauses. Defaults to true.
 */
export function useTimer(initialSeconds: number, running = true): number {
  const [elapsed, setElapsed] = useState(initialSeconds);

  // Sync elapsed when the backing value changes (e.g. new event from backend)
  useEffect(() => {
    setElapsed(initialSeconds);
  }, [initialSeconds]);

  useEffect(() => {
    if (!running) return;

    const id = setInterval(() => {
      setElapsed((prev) => prev + 1);
    }, 1000);

    return () => clearInterval(id);
  }, [running]);

  return elapsed;
}
