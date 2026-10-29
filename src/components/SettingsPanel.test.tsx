/**
 * SettingsPanel.test.tsx — unit tests for the SettingsPanel component.
 *
 * Test Coverage:
 * - Tab bar renders all 5 tabs with first tab (Profiles) active by default
 * - Save button invokes save_settings with the AppConfig argument expected by Rust
 * - Back/Cancel does NOT invoke save_settings
 * - Inverted calibration (sitting_mm >= standing_mm) shows validation error and disables Save
 * - Notifications tab exposes the existing effective communication-profile editor
 * - Tab switching shows correct content per tab
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import SettingsPanel from "./SettingsPanel";

// setup.ts already provides vi.mock for @tauri-apps/api/core and @tauri-apps/api/event

const mockOnClose = vi.fn();

const MOCK_SETTINGS = {
  sitting_mm: 750,
  standing_mm: 1050,
  desk_thickness_mm: 30,
  active_widget: "one-bar",
  timeline_skin: "semantic",
  show_welcome_on_startup: true,
  show_activity_status: true,
  telemetry_enabled: false,
  telemetry_device_id: "",
  notify_webhook_enabled: false,
  notify_webhook_url: null,
  voice_ai_model: null,
  relay_enabled: false,
  relay_url: "https://relay.desk.zentala.io",
  remote_lan_enabled: true,
};

beforeEach(() => {
  mockOnClose.mockClear();
  vi.mocked(invoke).mockClear();
});

describe("SettingsPanel", () => {
  it("renders without crashing and shows Settings title", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    await waitFor(() => {
      expect(screen.getByText("Settings")).toBeInTheDocument();
    });
  });

  it("renders tab bar with 5 tabs", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    await waitFor(() => {
      expect(screen.getByText("Profiles")).toBeInTheDocument();
      expect(screen.getByText("Calibr.")).toBeInTheDocument();
      expect(screen.getByText("Notif.")).toBeInTheDocument();
      expect(screen.getByText("More")).toBeInTheDocument();
      expect(screen.getByText("Debug")).toBeInTheDocument();
    });
  });

  it("shows Profiles tab content by default", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    await waitFor(() => {
      expect(screen.getByText("Ergonomic Profile")).toBeInTheDocument();
      expect(screen.queryByText("Communication Profile")).not.toBeInTheDocument();
      expect(screen.getByText(/Changes here are drafts until Save/i)).toBeInTheDocument();
      expect(screen.getByText(/Profile changes apply immediately/i)).toBeInTheDocument();
    });
  });

  it("switches to Calibration tab on click", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    await waitFor(() => {
      expect(screen.getByText("Ergonomic Profile")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Calibr."));
    await waitFor(() => {
      expect(screen.queryByText("Ergonomic Profile")).not.toBeInTheDocument();
    });
  });

  it("invokes save_settings with the AppConfig argument on Save", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return MOCK_SETTINGS;
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");

    const saveBtn = await screen.findByText("Save");
    fireEvent.click(saveBtn);

    await waitFor(() => {
      const calls = vi.mocked(invoke).mock.calls;
      const saveCall = calls.find((c) => c[0] === "save_settings");
      expect(saveCall).toBeDefined();
      const payload = saveCall![1] as { config?: Record<string, unknown>; settings?: unknown };
      expect(payload.config).toEqual(MOCK_SETTINGS);
      expect(payload).not.toHaveProperty("settings");
    });
  });

  it("notifies the widget of the activity preference only after Save succeeds", async () => {
    const dispatch = vi.spyOn(window, "dispatchEvent");
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return MOCK_SETTINGS;
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "More" }));
    fireEvent.click(screen.getByLabelText(/Show keyboard\/mouse activity/));
    expect(dispatch.mock.calls.some(([event]) => event.type === "moveup:settings-saved")).toBe(false);
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      const savedEvent = dispatch.mock.calls.map(([event]) => event)
        .find((event) => event.type === "moveup:settings-saved") as CustomEvent | undefined;
      expect(savedEvent?.detail).toEqual({ show_activity_status: false });
    });
    dispatch.mockRestore();
  });

  it("explains that the timeline theme applies immediately while other settings remain drafts", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "More" }));
    expect(await screen.findByText(/Theme changes apply immediately/i)).toBeInTheDocument();
    expect(screen.getByText(/Changes here are drafts until Save/i)).toBeInTheDocument();
  });

  it("shows optional Health guidance and reachable existing guides in More", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "More" }));

    expect(await screen.findByRole("heading", { name: "Optional health readings" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Privacy guide" })).toHaveAttribute(
      "href", "https://github.com/zentala/MoveUp/blob/main/docs/PRIVACY.md",
    );
    expect(screen.getByRole("link", { name: "Remote Display guide" })).toHaveAttribute(
      "href", "https://github.com/zentala/MoveUp/blob/main/docs/REMOTE_DISPLAY.md",
    );
  });

  it("keeps settings open and reports the backend error when saving fails", async () => {
    const dispatch = vi.spyOn(window, "dispatchEvent");
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "save_settings") throw new Error("store unavailable");
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(await screen.findByText("Save"));

    expect(await screen.findByText("Error: store unavailable")).toBeInTheDocument();
    expect(mockOnClose).not.toHaveBeenCalled();
    expect(dispatch.mock.calls.some(([event]) => event.type === "moveup:settings-saved")).toBe(false);
    dispatch.mockRestore();
  });

  it("keeps defaults out of a failed load and retries before enabling Save", async () => {
    let loadAttempts = 0;
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") {
        loadAttempts += 1;
        if (loadAttempts === 1) throw new Error("settings unavailable");
        return MOCK_SETTINGS;
      }
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("settings unavailable");
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
    expect(screen.queryByLabelText("Sitting height (mm)")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Retry" }));

    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "Calibr." }));
    expect(await screen.findByLabelText("Sitting height (mm)")).toHaveValue(750);
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
    expect(loadAttempts).toBe(2);
  });

  it("clears corrected calibration validation and saves the corrected draft", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return { ...MOCK_SETTINGS, sitting_mm: 900, standing_mm: 900 };
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "Calibr." }));
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    const sitting = screen.getByLabelText("Sitting height (mm)");
    expect(await screen.findByText("Standing height must be greater than sitting height")).toBeInTheDocument();
    expect(sitting).toHaveAttribute("aria-invalid", "true");
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();

    fireEvent.change(sitting, { target: { value: "700" } });

    expect(screen.queryByText("Standing height must be greater than sitting height")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mockOnClose).toHaveBeenCalledTimes(1));
    const saveCall = vi.mocked(invoke).mock.calls.find((call) => call[0] === "save_settings");
    expect((saveCall?.[1] as { config: typeof MOCK_SETTINGS }).config).toMatchObject({ sitting_mm: 700, standing_mm: 900 });
  });

  it("associates a sitting-only range error with sitting and clears it after correction", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return { ...MOCK_SETTINGS, sitting_mm: 390 };
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "Calibr." }));

    const sitting = screen.getByLabelText("Sitting height (mm)");
    const standing = screen.getByLabelText("Standing height (mm)");
    const error = await screen.findByRole("alert");
    expect(error).toHaveTextContent("Sitting height must be between 400 and 900 mm");
    expect(sitting).toHaveAttribute("aria-invalid", "true");
    expect(sitting).toHaveAttribute("aria-describedby", "calibration-validation-error");
    expect(standing).toHaveAttribute("aria-invalid", "false");
    expect(standing).not.toHaveAttribute("aria-describedby");
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();

    fireEvent.change(sitting, { target: { value: "750" } });

    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(sitting).toHaveAttribute("aria-invalid", "false");
    expect(sitting).not.toHaveAttribute("aria-describedby");
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mockOnClose).toHaveBeenCalledTimes(1));
    const saveCall = vi.mocked(invoke).mock.calls.find((call) => call[0] === "save_settings");
    expect((saveCall?.[1] as { config: typeof MOCK_SETTINGS }).config).toMatchObject({ sitting_mm: 750, standing_mm: 1050 });
  });

  it("associates a standing-only range error with standing and clears it after correction", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return { ...MOCK_SETTINGS, standing_mm: 1500 };
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "Calibr." }));

    const sitting = screen.getByLabelText("Sitting height (mm)");
    const standing = screen.getByLabelText("Standing height (mm)");
    const error = await screen.findByRole("alert");
    expect(error).toHaveTextContent("Standing height must be between 900 and 1400 mm");
    expect(standing).toHaveAttribute("aria-invalid", "true");
    expect(standing).toHaveAttribute("aria-describedby", "calibration-validation-error");
    expect(sitting).toHaveAttribute("aria-invalid", "false");
    expect(sitting).not.toHaveAttribute("aria-describedby");
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();

    fireEvent.change(standing, { target: { value: "1100" } });

    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(standing).toHaveAttribute("aria-invalid", "false");
    expect(standing).not.toHaveAttribute("aria-describedby");
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mockOnClose).toHaveBeenCalledTimes(1));
    const saveCall = vi.mocked(invoke).mock.calls.find((call) => call[0] === "save_settings");
    expect((saveCall?.[1] as { config: typeof MOCK_SETTINGS }).config).toMatchObject({ sitting_mm: 750, standing_mm: 1100 });
  });

  it("describes both invalid calibration ranges and updates each field as it is corrected", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return { ...MOCK_SETTINGS, sitting_mm: 390, standing_mm: 1500 };
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "Calibr." }));

    const sitting = screen.getByLabelText("Sitting height (mm)");
    const standing = screen.getByLabelText("Standing height (mm)");
    const error = await screen.findByRole("alert");
    expect(error).toHaveTextContent("Sitting height must be between 400 and 900 mm");
    expect(error).toHaveTextContent("standing height must be between 900 and 1400 mm");
    expect(sitting).toHaveAttribute("aria-invalid", "true");
    expect(standing).toHaveAttribute("aria-invalid", "true");
    expect(sitting).toHaveAttribute("aria-describedby", "calibration-validation-error");
    expect(standing).toHaveAttribute("aria-describedby", "calibration-validation-error");
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();

    fireEvent.change(sitting, { target: { value: "750" } });

    expect(await screen.findByRole("alert")).toHaveTextContent("Standing height must be between 900 and 1400 mm");
    expect(sitting).toHaveAttribute("aria-invalid", "false");
    expect(sitting).not.toHaveAttribute("aria-describedby");
    expect(standing).toHaveAttribute("aria-invalid", "true");
    expect(standing).toHaveAttribute("aria-describedby", "calibration-validation-error");

    fireEvent.change(standing, { target: { value: "1100" } });

    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(sitting).toHaveAttribute("aria-invalid", "false");
    expect(sitting).not.toHaveAttribute("aria-describedby");
    expect(standing).toHaveAttribute("aria-invalid", "false");
    expect(standing).not.toHaveAttribute("aria-describedby");
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(mockOnClose).toHaveBeenCalledTimes(1));
    const saveCall = vi.mocked(invoke).mock.calls.find((call) => call[0] === "save_settings");
    expect((saveCall?.[1] as { config: typeof MOCK_SETTINGS }).config).toMatchObject({ sitting_mm: 750, standing_mm: 1100 });
  });

  it("prevents duplicate saves and retains the corrected draft after a rejected save", async () => {
    let failSave!: (reason: Error) => void;
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") return Promise.resolve(MOCK_SETTINGS);
      if (cmd === "save_settings") return new Promise((_, reject) => { failSave = reject; });
      return Promise.resolve(null);
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "Calibr." }));
    const sitting = await screen.findByLabelText("Sitting height (mm)");
    fireEvent.change(sitting, { target: { value: "740" } });
    const save = screen.getByRole("button", { name: "Save" });
    fireEvent.click(save);
    fireEvent.click(save);

    await waitFor(() => expect(save).toBeDisabled());
    expect(vi.mocked(invoke).mock.calls.filter((call) => call[0] === "save_settings")).toHaveLength(1);
    failSave!(new Error("write failed"));

    expect(await screen.findByText("Error: write failed")).toBeInTheDocument();
    expect(screen.getByLabelText("Sitting height (mm)")).toHaveValue(740);
    expect(mockOnClose).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
  });

  it("does NOT invoke save_settings when Back button clicked", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);

    const backBtn = await screen.findByLabelText("Back");
    fireEvent.click(backBtn);

    const saveSettingsCalls = vi.mocked(invoke).mock.calls.filter(
      (c) => c[0] === "save_settings"
    );
    expect(saveSettingsCalls).toHaveLength(0);
    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it("shows validation error and disables Save when sitting_mm >= standing_mm", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") {
        return { ...MOCK_SETTINGS, sitting_mm: 900, standing_mm: 900 };
      }
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);

    await screen.findByText("Ergonomic Profile");
    fireEvent.click(screen.getByRole("tab", { name: "Calibr." }));
    const saveBtn = screen.getByRole("button", { name: "Save" });

    await waitFor(() => {
      expect(
        screen.getByText("Standing height must be greater than sitting height")
      ).toBeInTheDocument();
    });

    expect(saveBtn).toBeDisabled();

    const saveSettingsCalls = vi.mocked(invoke).mock.calls.filter(
      (c) => c[0] === "save_settings"
    );
    expect(saveSettingsCalls).toHaveLength(0);
  });

  it("shows the existing communication-profile editor on the Notifications tab", async () => {
    let activeCommunicationId = "default";
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "list_communication_profiles") {
        return [
          { id: "default", name: "Default", description: "Default notices", path: "default.json" },
          { id: "gentle", name: "Gentle", description: "Gentle notices", path: "gentle.json" },
        ];
      }
      if (cmd === "get_active_profiles") {
        return {
          communication_id: activeCommunicationId,
          ergonomic_id: "standard",
          effective_limits: {
            sitting_secs: 2400, standing_secs: 60, standing_target_secs: 900,
            standing_max_secs: 5400, break_min_secs: 60, break_credit_multiplier: 2,
            day_break_min_secs: 21600, posture_balance_min_sitting_secs: 21600,
            max_continuous_computer_secs: 3600, computer_break_reset_secs: 300,
          },
          session_limit_secs: 2400, stand_limit_secs: 900, notifications_enabled: true,
          dismiss_durations_mins: [5, 15, 30], current_dismiss_minutes: 5,
          snooze_minutes: 5, tone_shift_after_dismisses: 3, notify_cooldowns_secs: [0, 300, 900],
        };
      }
      if (cmd === "switch_communication_profile") {
        activeCommunicationId = "gentle";
        return null;
      }
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);

    // Switch to Notif. tab
    const notifTab = await screen.findByText("Notif.");
    fireEvent.click(notifTab);

    await waitFor(() => {
      expect(screen.getByText("Communication Profile")).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Edit JSON" })).toBeEnabled();
      expect(screen.getByRole("button", { name: "Duplicate" })).toBeEnabled();
      expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
    });

    fireEvent.change(screen.getByLabelText("Active profile"), { target: { value: "gentle" } });
    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("switch_communication_profile", { id: "gentle" });
    });

    fireEvent.click(screen.getByRole("button", { name: "Edit JSON" }));
    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("open_profile_in_editor", { path: "gentle.json" });
    });
  });

  it("toolbar buttons stay visible across all tabs", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);

    // Check toolbar on Time tab (default)
    await screen.findByText("Ergonomic Profile");
    expect(await screen.findByText("Save")).toBeInTheDocument();
    expect(screen.getByLabelText("Back")).toBeInTheDocument();

    // Switch to More tab
    fireEvent.click(screen.getByText("More"));
    expect(screen.getByText("Save")).toBeInTheDocument();
    expect(screen.getByLabelText("Back")).toBeInTheDocument();
    expect(await screen.findByText("Remote access")).toBeInTheDocument();
    expect(screen.getByText("LAN display · same network")).toBeInTheDocument();
    expect(screen.getByText(/This build does not upload usage data/)).toBeInTheDocument();
  });
});
