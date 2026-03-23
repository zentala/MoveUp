/**
 * SettingsPanel.test.tsx — unit tests for the SettingsPanel component.
 *
 * Test Coverage:
 * - Tab bar renders all 4 tabs with first tab active by default
 * - Save button invokes save_settings with correct field names (sit_limit_mins, etc.)
 * - Back/Cancel does NOT invoke save_settings
 * - Inverted calibration (sitting_mm >= standing_mm) shows validation error and disables Save
 * - Slider values are within expected range bounds
 * - Notification toggles render with correct default values (via tab switch)
 * - Tab switching shows correct content per tab
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import SettingsPanel from "./SettingsPanel";

// setup.ts already provides vi.mock for @tauri-apps/api/core and @tauri-apps/api/event

const mockOnClose = vi.fn();

const MOCK_SETTINGS = {
  sit_limit_mins: 40,
  stand_limit_mins: 15,
  sitting_mm: 750,
  standing_mm: 1050,
  notify_inactivity: true,
  notify_daily_posture_balance: true,
  notify_praise_halfway: false,
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

  it("renders tab bar with 4 tabs", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    await waitFor(() => {
      expect(screen.getByText("Time")).toBeInTheDocument();
      expect(screen.getByText("Calibr.")).toBeInTheDocument();
      expect(screen.getByText("Notif.")).toBeInTheDocument();
      expect(screen.getByText("More")).toBeInTheDocument();
    });
  });

  it("shows Time tab content by default", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    const slider = await screen.findByLabelText("Remind me to stand after (minutes)");
    expect(slider).toBeInTheDocument();
  });

  it("switches to Calibration tab on click", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    await screen.findByText("Time Limits");

    fireEvent.click(screen.getByText("Calibr."));
    await waitFor(() => {
      expect(screen.queryByText("Time Limits")).not.toBeInTheDocument();
    });
  });

  it("invokes save_settings with correct field names on Save", async () => {
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
      const payload = saveCall![1] as { settings: Record<string, unknown> };
      expect(payload.settings).toHaveProperty("sit_limit_mins");
      expect(payload.settings).toHaveProperty("stand_limit_mins");
      expect(payload.settings).toHaveProperty("sitting_mm");
      expect(payload.settings).toHaveProperty("standing_mm");
      // Must NOT contain old wrong field names
      expect(payload.settings).not.toHaveProperty("sitting_limit_minutes");
      expect(payload.settings).not.toHaveProperty("standing_limit_minutes");
      expect(payload.settings).not.toHaveProperty("sitting_height_mm");
      expect(payload.settings).not.toHaveProperty("standing_height_mm");
    });
  });

  it("does NOT invoke save_settings when Back button clicked", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);

    const backBtn = await screen.findByText("Back");
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

  it("sitting limit slider is within 10-90 range", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    const slider = await screen.findByLabelText("Remind me to stand after (minutes)");
    expect(slider).toHaveAttribute("min", "10");
    expect(slider).toHaveAttribute("max", "90");
    expect(slider).toHaveAttribute("step", "5");
  });

  it("standing limit slider is within 5-60 range", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);
    const slider = await screen.findByLabelText("Remind me to sit after (minutes)");
    expect(slider).toHaveAttribute("min", "5");
    expect(slider).toHaveAttribute("max", "60");
    expect(slider).toHaveAttribute("step", "5");
  });

  it("notification toggles render with correct values after switching tab", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") {
        return {
          ...MOCK_SETTINGS,
          notify_inactivity: true,
          notify_daily_posture_balance: false,
          notify_praise_halfway: true,
        };
      }
      return null;
    });

    render(<SettingsPanel onClose={mockOnClose} />);

    // Switch to Notif. tab
    const notifTab = await screen.findByText("Notif.");
    fireEvent.click(notifTab);

    await waitFor(() => {
      const checkboxes = screen.getAllByRole("checkbox");
      // notify_inactivity = true
      expect(checkboxes[0]).toBeChecked();
      // notify_daily_posture_balance = false
      expect(checkboxes[1]).not.toBeChecked();
      // notify_praise_halfway = true
      expect(checkboxes[2]).toBeChecked();
    });
  });

  it("footer buttons stay visible across all tabs", async () => {
    render(<SettingsPanel onClose={mockOnClose} />);

    // Check footer on Time tab (default)
    expect(await screen.findByText("Save")).toBeInTheDocument();
    expect(screen.getByText("Back")).toBeInTheDocument();

    // Switch to More tab
    fireEvent.click(screen.getByText("More"));
    expect(screen.getByText("Save")).toBeInTheDocument();
    expect(screen.getByText("Back")).toBeInTheDocument();
  });
});
