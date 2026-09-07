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

  it("shows the desk-offline overlay when the relay is up but the desk is not", () => {
    render(
      <ConnectionOverlay wsConnected={true} sensorConnected={true} deskOnline={false} />,
    );
    const overlay = screen.getByTestId("conn-overlay-desk-offline");
    expect(overlay.textContent).toContain("Desk offline");
    expect(overlay.textContent).toContain("Showing last known state");
    // "Reconnecting…" would be a lie: this phone's socket is fine.
    expect(screen.queryByTestId("conn-overlay-reconnecting")).toBeNull();
  });

  it("names the time the desk was last seen once it has been online", () => {
    const { rerender } = render(
      <ConnectionOverlay wsConnected={true} sensorConnected={true} deskOnline={true} />,
    );
    rerender(
      <ConnectionOverlay wsConnected={true} sensorConnected={true} deskOnline={false} />,
    );
    expect(screen.getByTestId("conn-overlay-desk-offline").textContent).toMatch(
      /Desk offline since \d\d:\d\d/,
    );
  });

  it("prefers the reconnecting overlay when the phone itself is disconnected", () => {
    render(
      <ConnectionOverlay wsConnected={false} sensorConnected={true} deskOnline={false} />,
    );
    expect(screen.getByTestId("conn-overlay-reconnecting")).toBeDefined();
    expect(screen.queryByTestId("conn-overlay-desk-offline")).toBeNull();
  });

  it("defaults deskOnline to true so LAN callers keep three states", () => {
    const { container } = render(
      <ConnectionOverlay wsConnected={true} sensorConnected={true} />,
    );
    expect(container.innerHTML).toBe("");
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
