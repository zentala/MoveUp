/**
 * useHealth.test.ts — transport behaviour of the health hook.
 *
 * Desktop: reads the cached view over IPC, refreshes on demand, maps an IPC
 * rejection onto a transient error rather than throwing.
 * Remote: never invokes IPC; replays the last published snapshot and follows
 * subsequent ones.
 */
import { renderHook, act, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import type { HealthView } from "@/generated/HealthView";

const NOW = 1_715_000_000_000;

function view(overrides: Partial<HealthView> = {}): HealthView {
  return {
    configured: true,
    snapshot: { steps_today: 10, source_id: "google_fit", fetched_at_ms: NOW },
    ...overrides,
  };
}

/** Fresh module instance so `isTauriRuntime()` sees the current window. */
async function importHealth() {
  return import("./useHealth");
}

describe("useHealth on the desktop transport", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    vi.mocked(invoke).mockResolvedValue(view());
  });

  it("loads the cached view with get_health_today on mount", async () => {
    const { useHealth } = await importHealth();
    const { result } = renderHook(() => useHealth());

    await waitFor(() => expect(result.current.view).not.toBeNull());
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_health_today");
    expect(result.current.view?.snapshot?.steps_today).toBe(10);
    expect(result.current.canRefresh).toBe(true);
  });

  it("refresh() calls refresh_health_now and replaces the view", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(view())
      .mockResolvedValueOnce(
        view({
          snapshot: {
            steps_today: 555,
            source_id: "google_fit",
            fetched_at_ms: NOW,
          },
        }),
      );
    const { useHealth } = await importHealth();
    const { result } = renderHook(() => useHealth());
    await waitFor(() => expect(result.current.view).not.toBeNull());

    await act(async () => {
      await result.current.refresh();
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("refresh_health_now");
    expect(result.current.view?.snapshot?.steps_today).toBe(555);
  });

  it("maps an IPC rejection onto a transient error instead of throwing", async () => {
    vi.mocked(invoke)
      .mockResolvedValueOnce(view())
      .mockRejectedValueOnce(new Error("ipc down"));
    const { useHealth } = await importHealth();
    const { result } = renderHook(() => useHealth());
    await waitFor(() => expect(result.current.view).not.toBeNull());

    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.view?.error_kind).toBe("transient");
    expect(result.current.view?.error_message).toMatch(/ipc down/);
  });

  it("surfaces a failing get_health_today as a transient error", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("boom"));
    const { useHealth } = await importHealth();
    const { result } = renderHook(() => useHealth());

    await waitFor(() => expect(result.current.view?.error_kind).toBe("transient"));
  });

  it("ignores a second refresh while one is in flight", async () => {
    let release: (v: HealthView) => void = () => {};
    vi.mocked(invoke)
      .mockResolvedValueOnce(view())
      .mockImplementationOnce(
        () => new Promise<HealthView>((resolve) => { release = resolve; }),
      );
    const { useHealth } = await importHealth();
    const { result } = renderHook(() => useHealth());
    await waitFor(() => expect(result.current.view).not.toBeNull());

    let second: HealthView | null = null;
    await act(async () => {
      const first = result.current.refresh();
      second = await result.current.refresh();
      release(view());
      await first;
    });

    expect(second).toBeNull();
  });
});

describe("useHealth on the remote transport", () => {
  beforeEach(() => {
    vi.resetModules();
    vi.clearAllMocks();
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  afterEach(() => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  });

  it("never touches Tauri IPC and cannot refresh", async () => {
    const { useHealth } = await importHealth();
    const { result } = renderHook(() => useHealth());

    expect(result.current.canRefresh).toBe(false);
    await act(async () => {
      expect(await result.current.refresh()).toBeNull();
    });
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it("picks up snapshots published after mount", async () => {
    const { useHealth, publishRemoteHealth } = await importHealth();
    const { result } = renderHook(() => useHealth());
    expect(result.current.view).toBeNull();

    act(() => {
      publishRemoteHealth(
        view({
          snapshot: { steps_today: 42, source_id: "push", fetched_at_ms: NOW },
        }),
      );
    });

    expect(result.current.view?.snapshot?.steps_today).toBe(42);
    expect(result.current.view?.snapshot?.source_id).toBe("push");
  });

  it("replays the last snapshot to a widget mounted afterwards", async () => {
    const { useHealth, publishRemoteHealth, resetRemoteHealth } =
      await importHealth();
    resetRemoteHealth();
    publishRemoteHealth(
      view({
        snapshot: { steps_today: 7, source_id: "curl", fetched_at_ms: NOW },
      }),
    );

    const { result } = renderHook(() => useHealth());
    expect(result.current.view?.snapshot?.steps_today).toBe(7);
  });

  it("stops delivering to an unmounted consumer", async () => {
    const { useHealth, publishRemoteHealth, resetRemoteHealth } =
      await importHealth();
    resetRemoteHealth();
    const { result, unmount } = renderHook(() => useHealth());
    unmount();

    act(() => {
      publishRemoteHealth(view({ configured: false, snapshot: null }));
    });

    expect(result.current.view).toBeNull();
  });
});
