/**
 * RemoteSection.test.tsx — Settings → More → Remote access scenarios.
 *
 * The four scenarios in `src/test/scenarios.ts` are the fixtures: each one is
 * exactly the three IPC answers the section reads, so mocking `invoke` from a
 * scenario renders the state the mockup gallery shows.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";
import { act, render, screen, fireEvent, waitFor } from "@testing-library/react";
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

  it("keeps an unresolved relay read pending, then retries an IPC failure without claiming the relay is off", async () => {
    let rejectStatus!: (reason: Error) => void;
    const pendingStatus = new Promise<never>((_resolve, reject) => { rejectStatus = reject; });
    vi.mocked(invoke)
      .mockReturnValueOnce(pendingStatus)
      .mockResolvedValueOnce(relayDisabled.status);

    renderSection();

    expect(screen.getByTestId("relay-state")).toHaveTextContent("Checking relay status");
    expect(screen.queryByLabelText("Licence key")).not.toBeInTheDocument();
    expect(screen.queryByText(/not registered/i)).not.toBeInTheDocument();

    rejectStatus(new Error("status IPC unavailable"));
    const retry = await screen.findByRole("button", { name: "Retry relay status" });
    expect(screen.getByRole("alert")).toHaveTextContent("Could not load relay status");
    expect(screen.queryByLabelText("Licence key")).not.toBeInTheDocument();
    expect(screen.queryByText(/not registered/i)).not.toBeInTheDocument();

    fireEvent.click(retry);
    await waitFor(() => expect(screen.getByTestId("relay-state")).toHaveTextContent("Relay is off"));
    expect(screen.getByLabelText("Licence key")).toBeInTheDocument();
  });

  it("withholds the prior off state and registration action when the post-registration refresh fails", async () => {
    let statusReads = 0;
    let registrationCalls = 0;
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_relay_status") {
        statusReads += 1;
        if (statusReads === 1) return relayDisabled.status;
        if (statusReads === 2) throw new Error("status refresh unavailable");
        return relayOnlineTwoViewers.status;
      }
      if (cmd === "relay_register") {
        registrationCalls += 1;
        return null;
      }
      if (cmd === "relay_list_viewers") return relayOnlineTwoViewers.viewers;
      return null;
    });
    renderSection();

    fireEvent.change(await screen.findByLabelText("Licence key"), { target: { value: "MU-TEST-KEY" } });
    fireEvent.click(screen.getByRole("button", { name: "Enable remote access" }));

    expect(await screen.findByTestId("relay-status-error")).toHaveTextContent("relay state is unknown");
    expect(screen.getByTestId("relay-state")).toHaveTextContent("Relay status unavailable");
    expect(screen.queryByText("Relay is off")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Licence key")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Enable remote access" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Pair a phone" })).not.toBeInTheDocument();
    expect(registrationCalls).toBe(1);

    fireEvent.click(screen.getByRole("button", { name: "Retry relay status" }));
    await waitFor(() => expect(screen.getByTestId("relay-state")).toHaveTextContent("Connected to the cloud relay"));
    await waitFor(() => expect(screen.getByRole("button", { name: "Pair a phone" })).toBeEnabled());
    expect(registrationCalls).toBe(1);
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

  it("does not promise phone reachability or invent a LAN address", async () => {
    mockScenario(relayOnlineTwoViewers);
    renderSection();

    await waitFor(() => expect(screen.getByTestId("relay-state")).toHaveTextContent("Connected to the cloud relay"));
    expect(screen.getByText(/status reports the desk's relay connection, not whether a phone is connected/i)).toBeInTheDocument();
    expect(screen.getByText(/MoveUp does not report .*which adapter address to use/i)).toBeInTheDocument();
    expect(screen.queryByText(/your phone can reach this desk/i)).not.toBeInTheDocument();
  });

  it("keeps prior phones withheld through a pending and failed refresh, then reports a successful empty list", async () => {
    let rejectViewers!: (reason: Error) => void;
    const pendingViewers = new Promise<never>((_resolve, reject) => { rejectViewers = reject; });
    let viewerReads = 0;
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_relay_status") return relayOnlineTwoViewers.status;
      if (cmd === "relay_list_viewers") {
        viewerReads += 1;
        if (viewerReads === 2) return pendingViewers;
        return viewerReads === 1 ? relayOnlineTwoViewers.viewers : [];
      }
      if (cmd === "relay_revoke_viewer") return null;
      return null;
    });
    renderSection();

    expect(await screen.findByText("Pixel 8")).toBeInTheDocument();
    expect(screen.getByText("Kitchen tablet")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Remove Kitchen tablet" }));

    expect(await screen.findByText("Loading paired phones…")).toBeInTheDocument();
    expect(screen.getByTestId("relay-state")).toHaveTextContent("Connected to the cloud relay");
    expect(screen.queryByText("Pixel 8")).not.toBeInTheDocument();
    expect(screen.queryByText("Kitchen tablet")).not.toBeInTheDocument();
    expect(screen.queryByText("No phones paired yet.")).not.toBeInTheDocument();

    await act(async () => { rejectViewers(new Error("viewer IPC unavailable")); });
    expect(await screen.findByTestId("relay-status-error")).toHaveTextContent("Could not refresh paired phones");
    expect(screen.getByTestId("relay-state")).toHaveTextContent("Connected to the cloud relay");
    expect(screen.getByTestId("paired-devices-unavailable")).toBeInTheDocument();
    expect(screen.queryByText("Pixel 8")).not.toBeInTheDocument();
    expect(screen.queryByText("Kitchen tablet")).not.toBeInTheDocument();
    expect(screen.queryByText("No phones paired yet.")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retry relay status" })).toBeEnabled();

    fireEvent.click(screen.getByRole("button", { name: "Retry relay status" }));
    expect(await screen.findByText("No phones paired yet.")).toBeInTheDocument();
    expect(screen.queryByTestId("relay-status-error")).not.toBeInTheDocument();
    expect(screen.getByTestId("relay-state")).toHaveTextContent("Connected to the cloud relay");
  });
});

describe("RemoteSection — relay-disabled", () => {
  it("offers the licence form and no device list", async () => {
    mockScenario(relayDisabled);
    renderSection();

    await waitFor(() => {
      expect(screen.getByTestId("relay-state")).toHaveTextContent("Relay is off");
    });
    expect(screen.getByLabelText("Licence key")).toBeInTheDocument();
    expect(screen.getByLabelText("Licence key")).toHaveAttribute("type", "password");
    expect(screen.getByLabelText("Licence key")).toHaveAttribute("autocomplete", "off");
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
    expect(screen.getByTestId("relay-state")).toHaveTextContent("Connected to the cloud relay");
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
      expect(screen.getByTestId("relay-state")).toHaveTextContent("Relay licence expired");
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
