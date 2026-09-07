/**
 * storage.test.ts — the stored pairing record, including the paths where the
 * browser refuses to cooperate (E022-T08).
 */
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  RELAY_STORAGE_KEY,
  clearRelayRecord,
  hasRelayRecord,
  loadRelayRecord,
  saveRelayRecord,
  type RelayRecord,
} from "./storage";

const record: RelayRecord = {
  relay_url: "https://relay.desk.zentala.io",
  desk_id: "desk-1",
  viewer_id: "viewer-1",
  viewer_token: "mu_v_token",
  desk_name: "Office",
};

beforeEach(() => {
  localStorage.clear();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("relay record storage", () => {
  it("round-trips a record", () => {
    expect(saveRelayRecord(record)).toBe(true);
    expect(loadRelayRecord()).toEqual(record);
    expect(hasRelayRecord()).toBe(true);
  });

  it("reports no record when nothing was stored", () => {
    expect(loadRelayRecord()).toBeNull();
    expect(hasRelayRecord()).toBe(false);
  });

  it("treats an empty stored value as no record", () => {
    localStorage.setItem(RELAY_STORAGE_KEY, "");
    expect(loadRelayRecord()).toBeNull();
  });

  it("drops a record that is not JSON", () => {
    localStorage.setItem(RELAY_STORAGE_KEY, "{not json");
    expect(loadRelayRecord()).toBeNull();
    // Cleared, so the next load is not a repeat of the same failure.
    expect(localStorage.getItem(RELAY_STORAGE_KEY)).toBeNull();
  });

  it("drops a record missing a required field", () => {
    localStorage.setItem(
      RELAY_STORAGE_KEY,
      JSON.stringify({ ...record, viewer_token: undefined }),
    );
    expect(loadRelayRecord()).toBeNull();
    expect(localStorage.getItem(RELAY_STORAGE_KEY)).toBeNull();
  });

  it("survives a browser that throws on getItem", () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("SecurityError");
    });
    expect(loadRelayRecord()).toBeNull();
  });

  it("reports a refused write instead of throwing", () => {
    vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("QuotaExceededError");
    });
    expect(saveRelayRecord(record)).toBe(false);
  });

  it("clears without throwing when removeItem fails", () => {
    saveRelayRecord(record);
    vi.spyOn(Storage.prototype, "removeItem").mockImplementation(() => {
      throw new Error("SecurityError");
    });
    expect(() => clearRelayRecord()).not.toThrow();
  });

  it("forgets the desk on clear", () => {
    saveRelayRecord(record);
    clearRelayRecord();
    expect(hasRelayRecord()).toBe(false);
  });
});
