import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import StateIndicator from "./StateIndicator";

describe("StateIndicator", () => {
  it("shows state and height", () => {
    render(<StateIndicator state="Sitting" deskHeightCm={72} />);
    expect(screen.getByText("Sitting")).toBeTruthy();
    expect(screen.getByText("72 cm")).toBeTruthy();
  });

  it("labels recent keyboard or mouse input without claiming measured presence", () => {
    render(<StateIndicator state="Standing" deskHeightCm={105} idleSecs={10} showActivity={true} />);
    expect(screen.getByText("Recent input")).toBeTruthy();
  });

  it("labels input inactivity as a duration rather than measured absence", () => {
    render(<StateIndicator state="Standing" deskHeightCm={105} idleSecs={150} showActivity={true} />);
    expect(screen.getByText("No recent input · 2m 30s")).toBeTruthy();
  });

  it("hides height when Away", () => {
    render(<StateIndicator state="Away" deskHeightCm={105} idleSecs={300} showActivity={true} />);
    expect(screen.queryByText("105 cm")).toBeNull();
    expect(screen.getByText("No recent input · 5m 0s")).toBeTruthy();
  });

  it("hides activity when showActivity is false", () => {
    render(<StateIndicator state="Standing" deskHeightCm={105} idleSecs={150} showActivity={false} />);
    expect(screen.queryByText("Active")).toBeNull();
    expect(screen.queryByText(/Idle/)).toBeNull();
  });

  it("renders without activity props (backward compat)", () => {
    render(<StateIndicator state="Sitting" deskHeightCm={72} />);
    expect(screen.getByText("Sitting")).toBeTruthy();
    expect(screen.queryByText("Active")).toBeNull();
  });

  it("shows a source-qualified unknown label and omits height when no current value is available", () => {
    render(<StateIndicator state={null} stateLabel="Unknown" deskHeightCm={0} showHeight={false} />);
    expect(screen.getByText("Unknown")).toBeInTheDocument();
    expect(screen.queryByText(/cm/)).not.toBeInTheDocument();
  });
});
