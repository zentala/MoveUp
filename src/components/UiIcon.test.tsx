import { render, screen } from "@testing-library/react";
import { Settings } from "lucide-react";
import { describe, expect, it } from "vitest";
import { UiIcon } from "./UiIcon";

describe("UiIcon", () => {
  it("renders a decorative icon with the shared size and stroke", () => {
    const { container } = render(<UiIcon icon={Settings} />);
    const icon = container.querySelector("svg");
    expect(icon).toHaveAttribute("aria-hidden", "true");
    expect(icon).toHaveAttribute("width", "16");
    expect(icon).toHaveAttribute("stroke-width", "1.5");
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });
});
