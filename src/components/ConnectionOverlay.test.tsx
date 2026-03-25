/**
 * ConnectionOverlay.test.tsx — unit tests for the connection status overlay.
 */
import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ConnectionOverlay } from "@/components/ConnectionOverlay";

describe("ConnectionOverlay", () => {
  it("renders nothing when both WS and sensor are connected", () => {
    const { container } = render(
      <ConnectionOverlay wsConnected={true} sensorConnected={true} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("shows reconnecting overlay when WS is disconnected", () => {
    render(<ConnectionOverlay wsConnected={false} sensorConnected={true} />);
    const overlay = screen.getByTestId("conn-overlay-reconnecting");
    expect(overlay).toBeDefined();
    expect(overlay.textContent).toContain("Reconnecting...");
  });

  it("shows sensor banner when WS connected but sensor lost", () => {
    render(<ConnectionOverlay wsConnected={true} sensorConnected={false} />);
    const banner = screen.getByTestId("conn-overlay-sensor-lost");
    expect(banner).toBeDefined();
    expect(banner.textContent).toContain("Sensor disconnected");
  });

  it("shows reconnecting overlay (not sensor banner) when both disconnected", () => {
    render(<ConnectionOverlay wsConnected={false} sensorConnected={false} />);
    // WS disconnect takes priority — full overlay shown
    const overlay = screen.getByTestId("conn-overlay-reconnecting");
    expect(overlay).toBeDefined();
    // Sensor banner should NOT be present
    expect(screen.queryByTestId("conn-overlay-sensor-lost")).toBeNull();
  });

  it("shows last connected time on reconnecting overlay after a previous connection", () => {
    // First render as connected to set the time
    const { rerender } = render(
      <ConnectionOverlay wsConnected={true} sensorConnected={true} />,
    );
    // Then disconnect
    rerender(<ConnectionOverlay wsConnected={false} sensorConnected={true} />);
    const overlay = screen.getByTestId("conn-overlay-reconnecting");
    expect(overlay.textContent).toContain("Last connected:");
  });
});
