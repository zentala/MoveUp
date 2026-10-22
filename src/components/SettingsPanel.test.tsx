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

  it("keeps settings open and reports the backend error when saving fails", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "save_settings") throw new Error("store unavailable");
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);
    fireEvent.click(await screen.findByText("Save"));

    expect(await screen.findByText("Error: store unavailable")).toBeInTheDocument();
    expect(mockOnClose).not.toHaveBeenCalled();
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
        return { ...MOCK_SETTINGS, sitting_mm: 1050, standing_mm: 750 };
      }
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);

    const saveBtn = await screen.findByText("Save");
    fireEvent.click(saveBtn);

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
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "list_communication_profiles") {
        return [
          { id: "default", name: "Default", description: "Default notices", path: "default.json" },
          { id: "gentle", name: "Gentle", description: "Gentle notices", path: "gentle.json" },
        ];
      }
      if (cmd === "get_active_profiles") {
        return { communication_id: "default", ergonomic_id: "standard" };
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
    expect(await screen.findByText("Save")).toBeInTheDocument();
    expect(screen.getByLabelText("Back")).toBeInTheDocument();

    // Switch to More tab
    fireEvent.click(screen.getByText("More"));
    expect(screen.getByText("Save")).toBeInTheDocument();
    expect(screen.getByLabelText("Back")).toBeInTheDocument();
  });
});
