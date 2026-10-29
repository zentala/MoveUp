import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { HealthHelpSection } from "./HealthHelpSection";

afterEach(() => vi.unstubAllGlobals());

describe("HealthHelpSection", () => {
  it("explains the optional sources and their relationship to desk tracking", () => {
    render(<HealthHelpSection />);

    expect(screen.getByRole("heading", { name: "Optional health readings" })).toBeInTheDocument();
    expect(screen.getByText(/do not affect desk-state estimates or break credit/i)).toBeInTheDocument();
    expect(screen.getByText("Google Fit:")).toBeInTheDocument();
    expect(screen.getByText(/off by default/i)).toBeInTheDocument();
    expect(screen.getByText(/existing phone source/i)).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Privacy guide" })).toHaveAttribute(
      "href",
      "https://github.com/zentala/MoveUp/blob/main/docs/PRIVACY.md",
    );
    expect(screen.getByRole("link", { name: "Remote Display guide" })).toHaveAttribute(
      "href",
      "https://github.com/zentala/MoveUp/blob/main/docs/REMOTE_DISPLAY.md",
    );
    expect(screen.getByText(/No account or setup is required/i)).toBeInTheDocument();
  });

  it("opens either existing guide through the external-link mechanism", () => {
    const open = vi.fn();
    vi.stubGlobal("open", open);
    render(<HealthHelpSection />);

    fireEvent.click(screen.getByRole("link", { name: "Privacy guide" }));
    fireEvent.click(screen.getByRole("link", { name: "Remote Display guide" }));

    expect(open).toHaveBeenNthCalledWith(
      1,
      "https://github.com/zentala/MoveUp/blob/main/docs/PRIVACY.md",
      "_blank",
    );
    expect(open).toHaveBeenNthCalledWith(
      2,
      "https://github.com/zentala/MoveUp/blob/main/docs/REMOTE_DISPLAY.md",
      "_blank",
    );
  });
});
