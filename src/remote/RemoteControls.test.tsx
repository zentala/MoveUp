/**
 * RemoteControls.test.tsx — the control surface and its gate (E022-T09).
 *
 * The gate is the point of the first two cases: a LAN transport reports
 * `control: false` and must render nothing at all, not a disabled panel that
 * looks like a temporary fault.
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { RemoteControls } from "./RemoteControls";
import type { CommandOutcome, Transport } from "./transports/types";

/** A transport that records what was sent and answers on demand. */
function fakeTransport(
  control: boolean,
  reply: () => Promise<CommandOutcome> = async () => ({ ok: true, error: null }),
) {
  const sent: Array<{ name: string; args: Record<string, unknown> }> = [];
  const transport = {
    id: control ? "relay" : "lan",
    capabilities: { control },
    connect: vi.fn(),
    close: vi.fn(),
    onMessage: vi.fn(() => vi.fn()),
    onStatus: vi.fn(() => vi.fn()),
    sendCommand: control
      ? vi.fn(async (name: string, args: Record<string, unknown>) => {
          sent.push({ name, args });
          return reply();
        })
      : undefined,
  } as unknown as Transport;
  return { transport, sent };
}

describe("RemoteControls", () => {
  it("renders nothing on a transport without the control capability", () => {
    const { transport } = fakeTransport(false);
    render(<RemoteControls sitMin={40} standMin={15} transport={transport} />);
    expect(screen.queryByTestId("remote-controls")).toBeNull();
  });

  it("renders nothing when there is no transport at all", () => {
    render(<RemoteControls sitMin={40} standMin={15} transport={null} />);
    expect(screen.queryByTestId("remote-controls")).toBeNull();
  });

  it("sends ack_alert with no arguments", async () => {
    const { transport, sent } = fakeTransport(true);
    render(<RemoteControls sitMin={40} standMin={15} transport={transport} />);

    fireEvent.click(screen.getByTestId("rc-ack"));

    await waitFor(() => expect(sent).toHaveLength(1));
    expect(sent[0]).toEqual({ name: "ack_alert", args: {} });
  });

  it("nudges the sit limit by five minutes", async () => {
    const { transport, sent } = fakeTransport(true);
    render(<RemoteControls sitMin={40} standMin={15} transport={transport} />);

    fireEvent.click(screen.getByTestId("rc-sit+"));

    await waitFor(() => expect(sent).toHaveLength(1));
    expect(sent[0]).toEqual({ name: "set_limits", args: { sit_min: 45 } });
  });

  it("clamps a limit to the allowlist bounds instead of sending it", async () => {
    const { transport, sent } = fakeTransport(true);
    render(<RemoteControls sitMin={5} standMin={1} transport={transport} />);

    fireEvent.click(screen.getByTestId("rc-sit-"));

    await waitFor(() => expect(sent).toHaveLength(1));
    expect(sent[0]).toEqual({ name: "set_limits", args: { sit_min: 5 } });
  });

  it("switches a profile by kind and name", async () => {
    const { transport, sent } = fakeTransport(true);
    render(<RemoteControls sitMin={40} standMin={15} transport={transport} />);

    fireEvent.change(screen.getByTestId("rc-ergonomic"), { target: { value: "strict" } });

    await waitFor(() => expect(sent).toHaveLength(1));
    expect(sent[0]).toEqual({
      name: "switch_profile",
      args: { kind: "ergonomic", name: "strict" },
    });
  });

  it("shows a pending state until the result arrives", async () => {
    let resolve!: (outcome: CommandOutcome) => void;
    const { transport } = fakeTransport(
      true,
      () => new Promise<CommandOutcome>((r) => (resolve = r)),
    );
    render(<RemoteControls sitMin={40} standMin={15} transport={transport} />);

    fireEvent.click(screen.getByTestId("rc-ack"));

    await waitFor(() =>
      expect(screen.getByTestId("rc-ack")).toHaveTextContent("Dismissing…"),
    );
    expect(screen.getByTestId("rc-sit+")).toBeDisabled();

    resolve({ ok: true, error: null });
    await waitFor(() =>
      expect(screen.getByTestId("rc-ack")).toHaveTextContent("Dismiss alert"),
    );
  });

  it("shows the desk's message when a command fails", async () => {
    const { transport } = fakeTransport(true, async () => ({
      ok: false,
      error: { code: "bad_args", message: "sit_min must be 5..=240" },
    }));
    render(<RemoteControls sitMin={40} standMin={15} transport={transport} />);

    fireEvent.click(screen.getByTestId("rc-ack"));

    await waitFor(() =>
      expect(screen.getByTestId("rc-error")).toHaveTextContent("sit_min must be 5..=240"),
    );
  });
});
