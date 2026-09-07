/**
 * WebhookSection.test.tsx — unit tests for the webhook settings section.
 *
 * Test Coverage:
 * - Renders the toggle and URL field from settings
 * - Toggling calls onChange with the flag flipped
 * - Editing the URL calls onChange with the new value
 * - A null URL renders as an empty field, not the string "null"
 * - The URL field is disabled while the webhook is off
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import WebhookSection from "./WebhookSection";
import { DEFAULT_SETTINGS } from "./SettingsTypes";
import type { DeskSettings } from "./SettingsTypes";

function renderSection(overrides: Partial<DeskSettings> = {}) {
  const onChange = vi.fn();
  const settings = { ...DEFAULT_SETTINGS, ...overrides };
  render(<WebhookSection settings={settings} onChange={onChange} />);
  return { onChange, settings };
}

describe("WebhookSection", () => {
  it("renders the toggle and the URL field", () => {
    renderSection({
      notify_webhook_enabled: true,
      notify_webhook_url: "https://ntfy.sh/desk",
    });

    expect(screen.getByText("Send alerts to a webhook")).toBeInTheDocument();
    expect(screen.getByLabelText("Webhook URL")).toHaveValue(
      "https://ntfy.sh/desk",
    );
  });

  it("reports the flipped flag when the toggle is clicked", () => {
    const { onChange } = renderSection({ notify_webhook_enabled: false });

    fireEvent.click(screen.getByRole("checkbox"));

    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ notify_webhook_enabled: true }),
    );
  });

  it("reports the typed URL", () => {
    const { onChange } = renderSection({ notify_webhook_enabled: true });

    fireEvent.change(screen.getByLabelText("Webhook URL"), {
      target: { value: "https://ntfy.sh/moveup" },
    });

    expect(onChange).toHaveBeenCalledWith(
      expect.objectContaining({ notify_webhook_url: "https://ntfy.sh/moveup" }),
    );
  });

  it("renders a null URL as an empty field", () => {
    renderSection({ notify_webhook_enabled: true, notify_webhook_url: null });

    expect(screen.getByLabelText("Webhook URL")).toHaveValue("");
  });

  it("disables the URL field while the webhook is off", () => {
    renderSection({ notify_webhook_enabled: false });

    expect(screen.getByLabelText("Webhook URL")).toBeDisabled();
  });
});
