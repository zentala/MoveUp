/**
 * PairedDevicesList.test.tsx — the four shadow paths of one small list.
 *
 * happy (rows), empty (a sentence, not blank pixels), nil-ish fields
 * (`last_seen: null`), and the disabled pass while another call is in flight.
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import PairedDevicesList from "./PairedDevicesList";
import { relayOnlineTwoViewers } from "@/test/scenarios";

const VIEWERS = relayOnlineTwoViewers.viewers;

describe("PairedDevicesList", () => {
  it("renders one row per paired phone", () => {
    render(<PairedDevicesList viewers={VIEWERS} busy={false} onRevoke={vi.fn()} />);

    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText("Pixel 8")).toBeInTheDocument();
    expect(screen.getByText("online")).toBeInTheDocument();
  });

  it("says the list is empty rather than rendering nothing", () => {
    render(<PairedDevicesList viewers={[]} busy={false} onRevoke={vi.fn()} />);

    expect(screen.getByTestId("paired-devices-empty")).toHaveTextContent("No phones paired yet.");
    expect(screen.queryByTestId("paired-devices-list")).not.toBeInTheDocument();
  });

  it("handles a phone that has never connected", () => {
    const viewer = { ...VIEWERS[0], online: false, last_seen: null };
    render(<PairedDevicesList viewers={[viewer]} busy={false} onRevoke={vi.fn()} />);

    expect(screen.getByText("last seen never connected")).toBeInTheDocument();
  });

  it("passes the right viewer id to onRevoke", () => {
    const onRevoke = vi.fn();
    render(<PairedDevicesList viewers={VIEWERS} busy={false} onRevoke={onRevoke} />);

    fireEvent.click(screen.getByRole("button", { name: "Remove Pixel 8" }));

    expect(onRevoke).toHaveBeenCalledWith("v-phone");
  });

  it("disables every Remove button while busy", () => {
    render(<PairedDevicesList viewers={VIEWERS} busy onRevoke={vi.fn()} />);

    for (const button of screen.getAllByRole("button")) {
      expect(button).toBeDisabled();
    }
  });
});
