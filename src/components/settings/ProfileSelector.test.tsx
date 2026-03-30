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
      if (cmd === "get_active_profiles") return { communication_id: "gentle", ergonomic_id: "default" };
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

  it("shows Reset button only when a non-default profile is active", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "list_communication_profiles") return MOCK_PROFILES;
      if (cmd === "get_active_profiles") return { communication_id: "gentle", ergonomic_id: "default" };
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
      if (cmd === "get_active_profiles") return { communication_id: "default", ergonomic_id: "standard" };
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
      if (cmd === "get_active_profiles") return { communication_id: "default", ergonomic_id: "standard" };
      return null;
    });

    render(<ProfileSelector type="ergonomic" label="Ergonomic Profile" />);

    await waitFor(() => {
      const select = screen.getByRole("combobox") as HTMLSelectElement;
      expect(select.value).toBe("standard");
    });
  });
});
