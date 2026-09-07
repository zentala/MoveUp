/**
 * useRemoteCommand.ts — one command at a time, with its answer (E022-T09).
 *
 * A remote command is not fire-and-forget: it travels phone → relay → desk
 * and the verdict comes back as a `command_result`. Until it does, the button
 * has to say so, because on a phone the alternative is the user pressing it
 * again and sending the command twice.
 */
import { useCallback, useState } from "react";
import type { Transport } from "./transports/types";

/** What the controls need to render the outcome of the last command. */
export interface RemoteCommandState {
  /** Label of the control currently waiting for a result, or null. */
  pending: string | null;
  /** Message from the last failed command, cleared on the next attempt. */
  error: string | null;
  send(label: string, name: string, args: Record<string, unknown>): Promise<void>;
}

/**
 * Sends commands over a transport, serialised.
 *
 * @param transport - the wire, or null when there is none. A transport
 *   without `sendCommand` (the LAN one) can never send, and saying so by
 *   doing nothing is deliberate: the caller already gates on
 *   `capabilities.control`.
 */
export function useRemoteCommand(transport: Transport | null): RemoteCommandState {
  const [pending, setPending] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const send = useCallback(
    async (label: string, name: string, args: Record<string, unknown>) => {
      const sendCommand = transport?.sendCommand?.bind(transport);
      if (!sendCommand || pending) return;
      setPending(label);
      setError(null);
      const outcome = await sendCommand(name, args);
      setPending(null);
      if (!outcome.ok) setError(outcome.error?.message ?? "Command failed");
    },
    [transport, pending],
  );

  return { pending, error, send };
}
