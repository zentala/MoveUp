/**
 * ShareStats.test.tsx — unit tests for the "Share My Stats" modal.
 *
 * Test Coverage:
 * - Renders preview card stats derived from metrics
 * - Falls back to em-dash for missing metrics
 * - Copy button writes formatted text via Clipboard API and shows "Copied!"
 * - Twitter/Reddit buttons open the expected share URLs
 * - Clicking the overlay closes; clicking inside the modal does not
 */
import { describe, it, expect, vi, beforeEach, afterEach, beforeAll, afterAll } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { ShareStats } from "./ShareStats";
import type { MetricSnapshot } from "@/types";

const METRICS: MetricSnapshot[] = [
  {
    id: "standing_pct",
    label: "Standing",
    result: { value: 0.42, display: "42%", level: "yellow", is_personal_best: false },
  },
  {
    id: "longest_session",
    label: "Longest session",
    result: { value: 1800, display: "30m", level: "red", is_personal_best: true },
  },
];

const BASE_PROPS = {
  metrics: METRICS,
  todayChanges: 7,
  todayStandingSecs: 1200,
  todaySittingSecs: 3600,
  todayScore: 12.4,
  onClose: vi.fn(),
};

// jsdom has no native modal implementation. Actual focus/inert behavior is
// exercised in Chromium by the focused Share browser suite.
beforeAll(() => {
  Object.defineProperty(HTMLDialogElement.prototype, "showModal", {
    configurable: true, value(this: HTMLDialogElement) { this.setAttribute("open", ""); this.querySelector<HTMLElement>("[autofocus]")?.focus(); },
  });
  Object.defineProperty(HTMLDialogElement.prototype, "close", {
    configurable: true, value(this: HTMLDialogElement) { this.removeAttribute("open"); },
  });
});
afterAll(() => {
  Reflect.deleteProperty(HTMLDialogElement.prototype, "showModal");
  Reflect.deleteProperty(HTMLDialogElement.prototype, "close");
});

beforeEach(() => {
  BASE_PROPS.onClose.mockClear();
  vi.stubGlobal("open", vi.fn());
  Object.assign(navigator, {
    clipboard: { writeText: vi.fn().mockResolvedValue(undefined) },
  });
});

afterEach(() => {
  Reflect.deleteProperty(document, "execCommand");
  vi.unstubAllGlobals();
});

describe("ShareStats", () => {
  it("renders preview stats from metrics", () => {
    render(<ShareStats {...BASE_PROPS} />);
    expect(screen.getByText("42%")).toBeInTheDocument();
    expect(screen.getByText("30m")).toBeInTheDocument();
    expect(screen.getByText("7")).toBeInTheDocument();
    expect(screen.getByText("+12 pts")).toBeInTheDocument();
  });

  it("falls back to an em-dash when a metric is missing", () => {
    render(<ShareStats {...BASE_PROPS} metrics={[]} />);
    const dashes = screen.getAllByText("—");
    expect(dashes.length).toBeGreaterThan(0);
  });

  it("copies formatted share text and shows Copied! feedback", async () => {
    render(<ShareStats {...BASE_PROPS} />);
    fireEvent.click(screen.getByText(/Copy Text/));

    await waitFor(() => {
      expect(navigator.clipboard.writeText).toHaveBeenCalledWith(
        expect.stringContaining("Tracked by MoveUp"),
      );
    });
    expect(await screen.findByText(/Copied!/)).toBeInTheDocument();
  });

  it("opens a Twitter intent URL with the share text", () => {
    render(<ShareStats {...BASE_PROPS} />);
    fireEvent.click(screen.getByText(/Post on X/));

    expect(window.open).toHaveBeenCalledWith(
      expect.stringContaining("https://twitter.com/intent/tweet?text="),
      "_blank",
    );
  });

  it("opens a Reddit submit URL referencing MoveUp", () => {
    render(<ShareStats {...BASE_PROPS} />);
    fireEvent.click(screen.getByRole("button", { name: "Reddit" }));

    const url = decodeURIComponent(vi.mocked(window.open).mock.calls[0][0] as string);
    expect(url).toContain("https://reddit.com/submit?title=");
    expect(url).toContain("tracked by MoveUp");
  });

  it("closes when clicking the overlay but not the modal body", () => {
    render(<ShareStats {...BASE_PROPS} />);
    fireEvent.click(screen.getByText("Standing"));
    expect(BASE_PROPS.onClose).not.toHaveBeenCalled();

    fireEvent.click(screen.getByText("share my stats").closest(".share-overlay")!);
    expect(BASE_PROPS.onClose).toHaveBeenCalledTimes(1);
  });

  it("closes via the close button", () => {
    render(<ShareStats {...BASE_PROPS} />);
    fireEvent.click(screen.getByTitle("Close"));
    expect(BASE_PROPS.onClose).toHaveBeenCalledTimes(1);
  });

  it("uses a temporary textarea inside the modal when the clipboard API fails", async () => {
    vi.mocked(navigator.clipboard.writeText).mockRejectedValueOnce(new Error("Unavailable"));
    const copy = vi.fn(() => {
      const area = document.querySelector("dialog textarea") as HTMLTextAreaElement;
      expect(area.value).toContain("Score: +12 points");
      return true;
    });
    Object.defineProperty(document, "execCommand", { configurable: true, value: copy });
    render(<ShareStats {...BASE_PROPS} />);
    const button = screen.getByRole("button", { name: "Copy Text" });
    button.focus();
    fireEvent.click(button);
    expect(await screen.findByText("Copied!")).toBeInTheDocument();
    expect(copy).toHaveBeenCalledWith("copy");
    expect(document.querySelector("textarea")).toBeNull();
    expect(button).toHaveFocus();
  });

  it("shows a manual copy route after both copy methods fail and can retry", async () => {
    vi.mocked(navigator.clipboard.writeText).mockRejectedValueOnce(new Error("Unavailable"));
    Object.defineProperty(document, "execCommand", { configurable: true, value: vi.fn(() => false) });
    render(<ShareStats {...BASE_PROPS} />);

    fireEvent.click(screen.getByRole("button", { name: "Copy Text" }));

    expect(await screen.findByTestId("copy-error")).toHaveTextContent("Copy failed.");
    expect(screen.getByText("Automatic copy failed. Retry or select the text below and copy it.")).toBeInTheDocument();
    expect(screen.queryByText("Copied!")).not.toBeInTheDocument();
    const manualCopy = screen.getByRole("textbox", { name: "Share text to copy manually" }) as HTMLTextAreaElement;
    expect(manualCopy.value).toContain("Score: +12 points");
    fireEvent.focus(manualCopy);
    expect(manualCopy.selectionStart).toBe(0);
    expect(manualCopy.selectionEnd).toBe(manualCopy.value.length);

    vi.mocked(navigator.clipboard.writeText).mockResolvedValueOnce(undefined);
    fireEvent.click(screen.getByRole("button", { name: "Retry copy" }));
    expect(await screen.findByText("Copied!")).toBeInTheDocument();
    expect(screen.queryByTestId("copy-error")).not.toBeInTheDocument();
  });

  it("labels the modal and handles native Escape cancellation", () => {
    render(<ShareStats {...BASE_PROPS} />);
    const dialog = screen.getByRole("dialog", { name: "share my stats" });
    expect(dialog).toHaveAttribute("open");
    expect(screen.getByRole("button", { name: "Close" })).toHaveFocus();
    fireEvent(dialog, new Event("cancel", { bubbles: false, cancelable: true }));
    expect(BASE_PROPS.onClose).toHaveBeenCalledOnce();
  });
});
