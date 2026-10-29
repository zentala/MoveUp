import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import CalibrationSection from "./CalibrationSection";
import { DEFAULT_SETTINGS } from "./SettingsTypes";

describe("calibration setup help", () => {
  it("is discoverable in calibration and explains USB detection, real readings, and unknown history", () => {
    render(
      <CalibrationSection
        settings={{ ...DEFAULT_SETTINGS }}
        onChange={vi.fn()}
        validationError={null}
        sittingInvalid={false}
        standingInvalid={false}
      />,
    );

    expect(screen.getByRole("heading", { name: "Calibration" })).toBeInTheDocument();
    expect(screen.getByText("Sitting height (mm)")).toBeInTheDocument();
    expect(screen.getByText("Standing height (mm)")).toBeInTheDocument();

    const help = screen.getByText("Sensor and first reading help");
    expect(help).toBeInTheDocument();
    fireEvent.click(help);

    expect(screen.getByText(/Windows can detect the USB controller.*not a sensor reading/i)).toBeInTheDocument();
    expect(screen.getByText(/In Device Manager.*controller, not that the sensor is sending readings/i)).toBeInTheDocument();
    expect(screen.getByText(/Move the desk and confirm that the height reading in MoveUp changes/i)).toBeInTheDocument();
    expect(screen.getByText(/Missing readings stay unknown/i)).toBeInTheDocument();
    expect(screen.getByText(/Sitting balance includes break credit; history remains recorded elapsed time/i)).toBeInTheDocument();
  });
});
