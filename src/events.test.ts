import { describe, expect, it } from 'vitest';

// Loaded as text so the assertion below reads the real Rust module, not a copy.
import rustSource from '../src-tauri/src/desk_events.rs?raw';

import { DESK_EVENTS } from './events';

/** Parses `pub const NAME: &str = "value";` lines out of the Rust module. */
function readRustEventNames(): Record<string, string> {
  const pattern = /pub const (\w+): &str = "([^"]+)";/g;
  const found: Record<string, string> = {};
  for (const match of rustSource.matchAll(pattern)) {
    found[match[1]] = match[2];
  }
  return found;
}

describe('desk event names', () => {
  it('are namespaced and unique', () => {
    const values = Object.values(DESK_EVENTS);
    for (const value of values) {
      expect(value.startsWith('desk:')).toBe(true);
    }
    expect(new Set(values).size).toBe(values.length);
  });

  it('match the Rust constants one for one', () => {
    const rust = readRustEventNames();
    // An empty parse would make the comparison below vacuously pass.
    expect(Object.keys(rust).length).toBeGreaterThan(0);
    expect(rust).toEqual(DESK_EVENTS);
  });
});
