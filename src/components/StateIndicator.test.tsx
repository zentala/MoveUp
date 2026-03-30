import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import StateIndicator from "./StateIndicator";

describe("StateIndicator", () => {
  it("shows state and height", () => {
    render(<StateIndicator state="Sitting" deskHeightCm={72} />);
    expect(screen.getByText("Sitting")).toBeTruthy();
    expect(screen.getByText("72 cm")).toBeTruthy();
  });

  it("shows Active when idle < 30s and showActivity true", () => {
    render(<StateIndicator state="Standing" deskHeightCm={105} idleSecs={10} showActivity={true} />);
    expect(screen.getByText("Active")).toBeTruthy();
  });

  it("shows Idle duration when idle >= 30s", () => {
    render(<StateIndicator state="Standing" deskHeightCm={105} idleSecs={150} showActivity={true} />);
    expect(screen.getByText("Idle 2m 30s")).toBeTruthy();
  });

  it("hides height when Away", () => {
    render(<StateIndicator state="Away" deskHeightCm={105} idleSecs={300} showActivity={true} />);
    expect(screen.queryByText("105 cm")).toBeNull();
    expect(screen.getByText("Idle 5m 0s")).toBeTruthy();
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
});
