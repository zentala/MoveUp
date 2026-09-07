/**
 * emitter.ts — the two-line listener set both transports need (E022-T08).
 *
 * Extracted so `lan.ts` and `relay.ts` do not carry two private copies of the
 * same `Set.add` / `Set.delete` pair.
 */
import type { Unsubscribe } from "./types";

/** A synchronous fan-out to zero or more listeners. */
export class Emitter<T> {
  private readonly listeners = new Set<(value: T) => void>();

  /** Subscribes; the returned function unsubscribes. */
  on(listener: (value: T) => void): Unsubscribe {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /** Delivers to every current listener. */
  emit(value: T): void {
    for (const listener of [...this.listeners]) listener(value);
  }

  /** Drops every listener (transport teardown). */
  clear(): void {
    this.listeners.clear();
  }
}
