/**
 * ProfileSelector.test.tsx — unit tests for the ProfileSelector component.
 *
 * Test Coverage:
 * - Renders label and select element
 * - Lists profiles from IPC in the dropdown
 * - Shows active profile as selected value
 * - Switches profile on dropdown change (invokes correct IPC command)
 * - Renders active profile description
 * - Shows Reset button only when non-default profile is active
 * - Handles IPC error gracefully
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import ProfileSelector from "./ProfileSelector";

// setup.ts provides global vi.mock for @tauri-apps/api/core

const MOCK_PROFILES = [
  { id: "default", name: "Default", description: "Standard nudging", path: "/profiles/default.json" },
  { id: "gentle", name: "Gentle", description: "Soft reminders", path: "/profiles/gentle.json" },
];

const MOCK_ACTIVE_PROFILES = {
  communication_id: "default",
  ergonomic_id: "standard",
  effective_limits: {
    sitting_secs: 2400,
    standing_secs: 60,
    standing_target_secs: 900,
    standing_max_secs: 5400,
    break_min_secs: 60,
    break_credit_multiplier: 2,
    day_break_min_secs: 21600,
    posture_balance_min_sitting_secs: 21600,
    max_continuous_computer_secs: 3600,
    computer_break_reset_secs: 300,
  },
  session_limit_secs: 2400,
  stand_limit_secs: 900,
  notifications_enabled: true,
  dismiss_durations_mins: [5, 15, 30],
  current_dismiss_minutes: 5,
  snooze_minutes: 5,
  tone_shift_after_dismisses: 3,
  notify_cooldowns_secs: [0, 300, 900],
};

beforeEach(() => {
  vi.mocked(invoke).mockClear();
});

describe("ProfileSelector — communication type", () => {
  it("renders label and a select element", async () => {
    render(<ProfileSelector type="communication" label="Communication Profile" />);

    await waitFor(() => {
      expect(screen.getByText("Communication Profile")).toBeInTheDocument();
    });

    expect(screen.getByRole("combobox")).toBeInTheDocument();
    expect(screen.getByLabelText("Active profile")).toBeInTheDocument();
  });

  it("lists profiles from IPC in the dropdown", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return MOCK_ACTIVE_PROFILES;
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    await waitFor(() => {
      expect(screen.getByRole("option", { name: "Default" })).toBeInTheDocument();
      expect(screen.getByRole("option", { name: "Gentle" })).toBeInTheDocument();
    });
  });

  it("shows active profile as the selected dropdown value", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return { ...MOCK_ACTIVE_PROFILES, communication_id: "gentle", ergonomic_id: "default" };
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    await waitFor(() => {
      const select = screen.getByRole("combobox") as HTMLSelectElement;
      expect(select.value).toBe("gentle");
    });
  });

  it("invokes switch_communication_profile with correct id on change", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return MOCK_ACTIVE_PROFILES;
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    await waitFor(() => {
      expect(screen.getByRole("option", { name: "Gentle" })).toBeInTheDocument();
    });

    fireEvent.change(screen.getByRole("combobox"), { target: { value: "gentle" } });

    await waitFor(() => {
      const calls = vi.mocked(invoke).mock.calls;
      const switchCall = calls.find((c) => c[0] === "switch_communication_profile");
      expect(switchCall).toBeDefined();
      expect(switchCall![1]).toEqual({ id: "gentle" });
    });
  });

  it("renders the active profile description", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return MOCK_ACTIVE_PROFILES;
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    await waitFor(() => {
      expect(screen.getByText("Standard nudging")).toBeInTheDocument();
    });
  });

  it("shows the active communication profile's actual reminder schedule and action effects", async () => {
    const runtime = {
      ...MOCK_ACTIVE_PROFILES,
      communication_id: "default",
      dismiss_durations_mins: [4, 11, 23],
      current_dismiss_minutes: 11,
      snooze_minutes: 11,
      tone_shift_after_dismisses: 2,
      notify_cooldowns_secs: [0, 45, 90],
    };
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return runtime;
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    expect(await screen.findByText(/Current Dismiss delay: 11 min/i)).toBeInTheDocument();
    expect(screen.getByText(/Snooze uses 11 min without advancing the Dismiss step/i)).toBeInTheDocument();
    expect(screen.getByText(/Dismiss delay sequence: 4 min → 11 min → 23 min/i)).toBeInTheDocument();
    expect(screen.getByText(/Snooze uses 11 min without advancing the Dismiss step/i)).toBeInTheDocument();
    expect(screen.getByText(/Dismiss acknowledges the reminder and advances the step/i)).toBeInTheDocument();
  });

  it.each(["rejected", "null"] as const)(
    "clears old applied values after a successful switch when the runtime read is %s, then retries without switching again",
    async (refreshOutcome) => {
      let finishRefresh: ((value: typeof MOCK_ACTIVE_PROFILES | null) => void) | undefined;
      let failRefresh: ((reason: Error) => void) | undefined;
      const refresh = new Promise<typeof MOCK_ACTIVE_PROFILES | null>((resolve, reject) => {
        finishRefresh = resolve;
        failRefresh = reject;
      });
      const freshRuntime = {
        ...MOCK_ACTIVE_PROFILES,
        communication_id: "gentle",
        current_dismiss_minutes: 15,
        dismiss_durations_mins: [15, 30],
      };
      let activeReadCount = 0;
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === "list_communication_profiles") return MOCK_PROFILES;
        if (cmd === "switch_communication_profile") return undefined;
        if (cmd === "get_active_profiles") {
          activeReadCount += 1;
          if (activeReadCount === 1) return MOCK_ACTIVE_PROFILES;
          if (activeReadCount === 2) return refresh;
          return freshRuntime;
        }
        return null;
      });

      render(<ProfileSelector type="communication" label="Communication Profile" />);
      expect(await screen.findByText(/Current Dismiss delay: 5 min/i)).toBeInTheDocument();

      fireEvent.change(screen.getByRole("combobox"), { target: { value: "gentle" } });
      await waitFor(() => expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "switch_communication_profile")).toHaveLength(1));
      await waitFor(() => expect(activeReadCount).toBe(2));
      expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe("gentle");

      if (refreshOutcome === "null") finishRefresh!(null);
      else failRefresh!(new Error("runtime read failed after switch"));

      expect(await screen.findByRole("alert")).toBeInTheDocument();
      expect(screen.queryByText(/Current Dismiss delay: 5 min/i)).not.toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Retry reading current settings" })).toBeInTheDocument();
      expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe("gentle");

      fireEvent.click(screen.getByRole("button", { name: "Retry reading current settings" }));
      expect(await screen.findByText(/Current Dismiss delay: 15 min/i)).toBeInTheDocument();
      expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe("gentle");
      expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "switch_communication_profile")).toHaveLength(1);
      expect(activeReadCount).toBe(3);
    },
  );

  it("does not select the default profile when the initial runtime snapshot is unavailable", async () => {
    let activeReadCount = 0;
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") {
        activeReadCount += 1;
        return activeReadCount === 1 ? null : MOCK_ACTIVE_PROFILES;
      }
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    expect(await screen.findByRole("alert")).toBeInTheDocument();
    expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe("");
    expect(screen.queryByText(/Current Dismiss delay:/i)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Retry reading current settings" }));
    expect(await screen.findByText(/Current Dismiss delay: 5 min/i)).toBeInTheDocument();
    expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe("default");
    expect(activeReadCount).toBe(2);
  });

  it("keeps a retry read ahead of later profile switches so an older result cannot overwrite the new selection", async () => {
    let finishRetry: ((value: typeof MOCK_ACTIVE_PROFILES) => void) | undefined;
    const delayedRetry = new Promise<typeof MOCK_ACTIVE_PROFILES>((resolve) => {
      finishRetry = resolve;
    });
    let activeReadCount = 0;
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "switch_communication_profile") return undefined;
      if (cmd === "get_active_profiles") {
        activeReadCount += 1;
        if (activeReadCount === 1) return MOCK_ACTIVE_PROFILES;
        if (activeReadCount === 2) throw new Error("refresh needed");
        if (activeReadCount === 3) return delayedRetry;
        return MOCK_ACTIVE_PROFILES;
      }
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);
    expect(await screen.findByText(/Current Dismiss delay: 5 min/i)).toBeInTheDocument();

    fireEvent.change(screen.getByRole("combobox"), { target: { value: "gentle" } });
    expect(await screen.findByRole("alert")).toHaveTextContent("refresh needed");
    const select = screen.getByRole("combobox") as HTMLSelectElement;
    expect(select.value).toBe("gentle");

    fireEvent.click(screen.getByRole("button", { name: "Retry reading current settings" }));
    await waitFor(() => expect(activeReadCount).toBe(3));
    expect(select).toBeDisabled();
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "switch_communication_profile")).toHaveLength(1);

    finishRetry!({ ...MOCK_ACTIVE_PROFILES, communication_id: "gentle", current_dismiss_minutes: 15 });
    expect(await screen.findByText(/Current Dismiss delay: 15 min/i)).toBeInTheDocument();
    expect(select).toBeEnabled();

    fireEvent.change(select, { target: { value: "default" } });
    await waitFor(() => expect(select.value).toBe("default"));
    expect(await screen.findByText(/Current Dismiss delay: 5 min/i)).toBeInTheDocument();
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "switch_communication_profile")).toHaveLength(2);
    expect(activeReadCount).toBe(4);
  });

  it("serializes Reset behind a deferred retry read, then allows Reset after the read settles", async () => {
    let finishRetry: ((value: typeof MOCK_ACTIVE_PROFILES) => void) | undefined;
    const delayedRetry = new Promise<typeof MOCK_ACTIVE_PROFILES>((resolve) => {
      finishRetry = resolve;
    });
    let activeReadCount = 0;
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "switch_communication_profile") return undefined;
      if (cmd === "get_active_profiles") {
        activeReadCount += 1;
        if (activeReadCount === 1) return MOCK_ACTIVE_PROFILES;
        if (activeReadCount === 2) throw new Error("refresh needed");
        if (activeReadCount === 3) return delayedRetry;
        return MOCK_ACTIVE_PROFILES;
      }
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);
    expect(await screen.findByText(/Current Dismiss delay: 5 min/i)).toBeInTheDocument();
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "gentle" } });
    expect(await screen.findByRole("alert")).toHaveTextContent("refresh needed");
    fireEvent.click(screen.getByRole("button", { name: "Retry reading current settings" }));
    await waitFor(() => expect(activeReadCount).toBe(3));

    const reset = screen.getByRole("button", { name: "Reset" });
    const resetDisabledDuringRead = (reset as HTMLButtonElement).disabled;
    fireEvent.click(reset);
    const switchesWhileReadPending = vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "switch_communication_profile").length;
    finishRetry!({ ...MOCK_ACTIVE_PROFILES, communication_id: "gentle", current_dismiss_minutes: 15 });

    expect(await screen.findByText(/Current Dismiss delay: 15 min/i)).toBeInTheDocument();
    expect(resetDisabledDuringRead).toBe(true);
    expect(switchesWhileReadPending).toBe(1);
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    expect(await screen.findByText(/Current Dismiss delay: 5 min/i)).toBeInTheDocument();
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "switch_communication_profile")).toHaveLength(2);
    expect(activeReadCount).toBe(4);
  });

  it("guards Duplicate/Create Enter during a deferred retry, then creates and switches after it settles", async () => {
    let finishRetry: ((value: typeof MOCK_ACTIVE_PROFILES) => void) | undefined;
    const delayedRetry = new Promise<typeof MOCK_ACTIVE_PROFILES>((resolve) => {
      finishRetry = resolve;
    });
    let activeReadCount = 0;
    let duplicated = false;
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") {
        return duplicated ? [...MOCK_PROFILES, { id: "copy", name: "Copy", description: "Copied profile", path: "/profiles/copy.json" }] : MOCK_PROFILES;
      }
      if (cmd === "switch_communication_profile") return undefined;
      if (cmd === "duplicate_profile") {
        duplicated = true;
        return "copy";
      }
      if (cmd === "get_active_profiles") {
        activeReadCount += 1;
        if (activeReadCount === 1) return MOCK_ACTIVE_PROFILES;
        if (activeReadCount === 2) throw new Error("refresh needed");
        if (activeReadCount === 3) return delayedRetry;
        return activeReadCount === 4
          ? { ...MOCK_ACTIVE_PROFILES, communication_id: "gentle", current_dismiss_minutes: 15 }
          : { ...MOCK_ACTIVE_PROFILES, communication_id: "copy", current_dismiss_minutes: 15 };
      }
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);
    expect(await screen.findByText(/Current Dismiss delay: 5 min/i)).toBeInTheDocument();
    fireEvent.change(screen.getByRole("combobox"), { target: { value: "gentle" } });
    expect(await screen.findByRole("alert")).toHaveTextContent("refresh needed");
    fireEvent.click(screen.getByRole("button", { name: "Duplicate" }));
    const nameInput = screen.getByPlaceholderText("New profile name");
    fireEvent.change(nameInput, { target: { value: "Copy" } });
    fireEvent.click(screen.getByRole("button", { name: "Retry reading current settings" }));
    await waitFor(() => expect(activeReadCount).toBe(3));

    const createDisabledDuringRead = (screen.getByRole("button", { name: "Create" }) as HTMLButtonElement).disabled;
    fireEvent.keyDown(nameInput, { key: "Enter", code: "Enter", charCode: 13 });
    const duplicatesWhileReadPending = vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "duplicate_profile").length;
    finishRetry!({ ...MOCK_ACTIVE_PROFILES, communication_id: "gentle", current_dismiss_minutes: 15 });

    expect(await screen.findByText(/Current Dismiss delay: 15 min/i)).toBeInTheDocument();
    expect(createDisabledDuringRead).toBe(true);
    expect(duplicatesWhileReadPending).toBe(0);
    fireEvent.keyDown(nameInput, { key: "Enter", code: "Enter", charCode: 13 });
    await waitFor(() => expect((screen.getByRole("combobox") as HTMLSelectElement).value).toBe("copy"));
    expect(await screen.findByText(/Current Dismiss delay: 15 min/i)).toBeInTheDocument();
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "duplicate_profile")).toHaveLength(1);
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "switch_communication_profile")).toHaveLength(2);
    expect(activeReadCount).toBe(5);
  });

  it("shows Reset button only when a non-default profile is active", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return { ...MOCK_ACTIVE_PROFILES, communication_id: "gentle", ergonomic_id: "default" };
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Reset/i })).toBeInTheDocument();
    });
  });

  it("does NOT show Reset button when default profile is active", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return MOCK_ACTIVE_PROFILES; // communication_id: "default"
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    await waitFor(() => {
      expect(screen.queryByRole("button", { name: /Reset/i })).not.toBeInTheDocument();
    });
  });

  it("displays an error message when IPC fails", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") throw new Error("IPC error");
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    await waitFor(() => {
      expect(screen.getByText(/IPC error/)).toBeInTheDocument();
    });
  });
});

describe("ProfileSelector — ergonomic type", () => {
  it("invokes list_ergonomic_profiles and switch_ergonomic_profile", async () => {
    const ergoProfiles = [
      { id: "standard", name: "Standard", description: "Balanced ergonomics", path: "/profiles/standard.json" },
      { id: "aggressive", name: "Aggressive", description: "Frequent breaks", path: "/profiles/aggressive.json" },
    ];

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_ergonomic_profiles") return ergoProfiles;
      if (cmd === "get_active_profiles") return MOCK_ACTIVE_PROFILES;
      return null;
    });

    render(<ProfileSelector type="ergonomic" label="Ergonomic Profile" />);

    await waitFor(() => {
      expect(screen.getByRole("option", { name: "Standard" })).toBeInTheDocument();
      expect(screen.getByRole("option", { name: "Aggressive" })).toBeInTheDocument();
    });

    const calls = vi.mocked(invoke).mock.calls.map((c) => c[0]);
    expect(calls).toContain("list_ergonomic_profiles");

    fireEvent.change(screen.getByRole("combobox"), { target: { value: "aggressive" } });

    await waitFor(() => {
      const switchCall = vi.mocked(invoke).mock.calls.find(
        (c) => c[0] === "switch_ergonomic_profile"
      );
      expect(switchCall).toBeDefined();
      expect(switchCall![1]).toEqual({ id: "aggressive" });
    });
  });

  it("shows ergonomic active profile as selected", async () => {
    const ergoProfiles = [
      { id: "standard", name: "Standard", description: "Balanced ergonomics", path: "/profiles/standard.json" },
    ];

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_ergonomic_profiles") return ergoProfiles;
      if (cmd === "get_active_profiles") return MOCK_ACTIVE_PROFILES;
      return null;
    });

    render(<ProfileSelector type="ergonomic" label="Ergonomic Profile" />);

    await waitFor(() => {
      const select = screen.getByRole("combobox") as HTMLSelectElement;
      expect(select.value).toBe("standard");
    });
  });

  it("shows runtime ergonomic limits and distinct session overrides instead of catalog defaults", async () => {
    const ergoProfiles = [{
      id: "standard", name: "Standard", description: "Balanced ergonomics", path: "/profiles/standard.json",
    }];
    const runtime = {
      ...MOCK_ACTIVE_PROFILES,
      ergonomic_id: "standard",
      effective_limits: {
        ...MOCK_ACTIVE_PROFILES.effective_limits,
        sitting_secs: 2400,
        standing_secs: 45,
        standing_target_secs: 840,
        standing_max_secs: 2700,
        break_min_secs: 90,
        break_credit_multiplier: 1.5,
        day_break_min_secs: 19800,
        posture_balance_min_sitting_secs: 18000,
        max_continuous_computer_secs: 3300,
        computer_break_reset_secs: 420,
      },
      session_limit_secs: 3300,
      stand_limit_secs: 720,
    };
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_ergonomic_profiles") return ergoProfiles;
      if (cmd === "get_active_profiles") return runtime;
      return null;
    });

    render(<ProfileSelector type="ergonomic" label="Ergonomic Profile" />);

    expect(await screen.findByText(/Session engine sitting limit: 40 min; current sitting alert threshold: 55 min/)).toBeInTheDocument();
    expect(screen.queryByText(/minimum standing to count as a break/i)).not.toBeInTheDocument();
    expect(screen.getByText(/Full-break target: 14 min; maximum standing: 45 min/)).toBeInTheDocument();
    expect(screen.getByText(/Break credit starts after 1 min 30 sec; configured credit multiplier: 1.5 \(engine applies a value from 0 to 10\)/)).toBeInTheDocument();
  });

  it("does not promise an unsupported Snooze delay while preserving the actual Dismiss step", async () => {
    const runtime = { ...MOCK_ACTIVE_PROFILES, current_dismiss_minutes: 0, snooze_minutes: null };
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return runtime;
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    expect(await screen.findByText(/Current Dismiss delay: 0 min/i)).toBeInTheDocument();
    expect(screen.getByText(/Snooze is unavailable for the current policy step because its configured duration is unsupported/i)).toBeInTheDocument();
  });

  it("reports an empty configured delay list without inventing a frontend fallback", async () => {
    const runtime = { ...MOCK_ACTIVE_PROFILES, current_dismiss_minutes: 7, dismiss_durations_mins: [] };
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return runtime;
      return null;
    });

    render(<ProfileSelector type="communication" label="Communication Profile" />);

    expect(await screen.findByText(/Current Dismiss delay: 7 min/i)).toBeInTheDocument();
    expect(screen.getByText(/Configured Dismiss delay sequence: none; the current policy value is shown above/i)).toBeInTheDocument();
    expect(screen.queryByText(/5 min fallback/i)).not.toBeInTheDocument();
  });
});
