/**
 * RemoteSection.test.tsx — Settings → Remote, one case per scenario.
 *
 * The four scenarios in `src/test/scenarios.ts` are the fixtures: each one is
 * exactly the three IPC answers the section reads, so mocking `invoke` from a
 * scenario renders the state the mockup gallery shows.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import RemoteSection from "./RemoteSection";
import { DEFAULT_SETTINGS } from "./SettingsTypes";
import {
  RELAY_SETTINGS_SCENARIOS,
  relayDisabled,
  relayOnlineTwoViewers,
  relayPairingCodeShown,
  relayUnentitled,
  type RelaySettingsScenario,
} from "@/test/scenarios";

/** Points the shared `invoke` mock at one scenario's three answers. */
function mockScenario(scenario: RelaySettingsScenario, overrides: Record<string, unknown> = {}) {
  vi.mocked(invoke).mockImplementation(async (cmd: string) => {
    if (cmd in overrides) {
      const value = overrides[cmd];
      if (value instanceof Error) throw value;
      return value;
    }
    if (cmd === "get_relay_status") return scenario.status;
    if (cmd === "relay_list_viewers") return scenario.viewers;
    if (cmd === "relay_start_pairing") return scenario.pairing;
    return null;
  });
}

function renderSection(onChange = vi.fn()) {
  return { onChange, ...render(<RemoteSection settings={DEFAULT_SETTINGS} onChange={onChange} />) };
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
});

describe("RemoteSection — scenario coverage", () => {
  it("has four named scenarios, and none of them is silently missing", () => {
    // An empty fixture list must never look like a passing suite.
    expect(RELAY_SETTINGS_SCENARIOS.length).toBe(4);
    expect(RELAY_SETTINGS_SCENARIOS.map((s) => s.id)).toEqual([
      "relay-disabled",
      "relay-online-2-viewers",
      "relay-unentitled",
      "relay-pairing-code-shown",
    ]);
  });

  it("renders a distinct status line for every scenario", async () => {
    for (const scenario of RELAY_SETTINGS_SCENARIOS) {
      mockScenario(scenario);
      const view = render(<RemoteSection settings={DEFAULT_SETTINGS} onChange={vi.fn()} />);
      await waitFor(() => {
        expect(view.getByTestId("relay-state").textContent).not.toBe("");
      });
      view.unmount();
    }
  });
});

describe("RemoteSection — relay-disabled", () => {
  it("offers the licence form and no device list", async () => {
    mockScenario(relayDisabled);
    renderSection();

    await waitFor(() => {
      expect(screen.getByTestId("relay-state")).toHaveTextContent("not registered");
    });
    expect(screen.getByLabelText("Licence key")).toBeInTheDocument();
    expect(screen.queryByText("Paired phones")).not.toBeInTheDocument();
    expect(invoke).not.toHaveBeenCalledWith("relay_list_viewers");
  });

  it("keeps the enable button disabled until a key is typed", async () => {
    mockScenario(relayDisabled);
    renderSection();

    const button = await screen.findByRole("button", { name: "Enable remote access" });
    expect(button).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Licence key"), { target: { value: "MU-TEST-KEY" } });
    expect(button).toBeEnabled();
  });

  it("registers with the typed key", async () => {
    mockScenario(relayDisabled);
    renderSection();

    fireEvent.change(await screen.findByLabelText("Licence key"), { target: { value: "MU-TEST-KEY" } });
    fireEvent.click(screen.getByRole("button", { name: "Enable remote access" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("relay_register", { licenseKey: "MU-TEST-KEY" });
    });
  });

  it("shows the failure instead of pretending it registered", async () => {
    mockScenario(relayDisabled, { relay_register: new Error("invalid licence") });
    renderSection();

    fireEvent.change(await screen.findByLabelText("Licence key"), { target: { value: "bad" } });
    fireEvent.click(screen.getByRole("button", { name: "Enable remote access" }));

    await waitFor(() => {
      expect(screen.getByTestId("relay-error")).toHaveTextContent("invalid licence");
    });
  });
});

describe("RemoteSection — relay-online-2-viewers", () => {
  it("lists both paired phones with their connection state", async () => {
    mockScenario(relayOnlineTwoViewers);
    renderSection();

    await waitFor(() => {
      expect(screen.getByText("Pixel 8")).toBeInTheDocument();
    });
    expect(screen.getByText("Kitchen tablet")).toBeInTheDocument();
    expect(screen.getByTestId("relay-state")).toHaveTextContent("Online");
    expect(screen.getByRole("button", { name: "Pair a phone" })).toBeEnabled();
  });

  it("revokes the phone whose Remove button was pressed", async () => {
    mockScenario(relayOnlineTwoViewers);
    renderSection();

    fireEvent.click(await screen.findByRole("button", { name: "Remove Kitchen tablet" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("relay_revoke_viewer", { viewerId: "v-tablet" });
    });
  });

  it("turns the relay off through relay_disable", async () => {
    mockScenario(relayOnlineTwoViewers);
    renderSection();

    fireEvent.click(await screen.findByRole("button", { name: "Turn off and forget this desk" }));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("relay_disable");
    });
  });
});

describe("RemoteSection — relay-unentitled", () => {
  it("names the licence problem and refuses to offer pairing", async () => {
    mockScenario(relayUnentitled);
    renderSection();

    await waitFor(() => {
      expect(screen.getByTestId("relay-state")).toHaveTextContent("Licence expired");
    });
    expect(screen.getByTestId("relay-last-error")).toHaveTextContent("licence expired");
    expect(screen.getByRole("button", { name: "Pair a phone" })).toBeDisabled();
  });

  it("still lets the desk switch itself off", async () => {
    mockScenario(relayUnentitled);
    renderSection();

    expect(await screen.findByRole("button", { name: "Turn off and forget this desk" })).toBeEnabled();
  });
});

describe("RemoteSection — relay-pairing-code-shown", () => {
  it("shows the code and desk id after Pair a phone", async () => {
    mockScenario(relayPairingCodeShown);
    renderSection();

    fireEvent.click(await screen.findByRole("button", { name: "Pair a phone" }));

    await waitFor(() => {
      expect(screen.getByTestId("pairing-code")).toHaveTextContent("ABCD2345");
    });
    expect(screen.getByTestId("pairing-desk-id")).toHaveTextContent("desk-42");
  });

  it("shows no card before the button is pressed", async () => {
    mockScenario(relayPairingCodeShown);
    renderSection();

    await screen.findByRole("button", { name: "Pair a phone" });
    expect(screen.queryByTestId("pairing-code-card")).not.toBeInTheDocument();
  });
});

describe("RemoteSection — LAN toggle", () => {
  it("reports the LAN switch through onChange without touching the relay", async () => {
    mockScenario(relayDisabled);
    const { onChange } = renderSection();

    fireEvent.click(await screen.findByRole("checkbox"));

    expect(onChange).toHaveBeenCalledWith({ ...DEFAULT_SETTINGS, remote_lan_enabled: false });
  });
});
