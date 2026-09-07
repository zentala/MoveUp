/**
 * VoiceCapture.test.tsx — the four data paths plus the double-click guard.
 *
 * happy  — token present, transcript sent, ack from the WS stream renders
 * nil    — no token in localStorage, the token sheet gates everything
 * empty  — whitespace-only transcript never reaches the network
 * error  — a non-OK response and a rejected fetch both surface a message
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { VoiceCapture } from "./VoiceCapture";
import { emitVoiceAck } from "@/hooks/useRemoteDesk";

const fetchMock = vi.fn();

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  vi.stubGlobal("fetch", fetchMock);
  fetchMock.mockResolvedValue({ ok: true, status: 200 });
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("VoiceCapture — token sheet (nil path)", () => {
  it("shows the token sheet and no textarea when no token is stored", () => {
    render(<VoiceCapture />);
    expect(screen.getByTestId("voice-token-sheet")).toBeInTheDocument();
    expect(screen.queryByTestId("voice-input")).not.toBeInTheDocument();
  });

  it("stores the token in localStorage and reveals the textarea", () => {
    render(<VoiceCapture />);
    fireEvent.change(screen.getByLabelText(/display token/i), {
      target: { value: "s3cret" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save token/i }));

    expect(localStorage.getItem("desk_token")).toBe("s3cret");
    expect(screen.getByTestId("voice-input")).toBeInTheDocument();
  });

  it("ignores a blank token submission", () => {
    render(<VoiceCapture />);
    fireEvent.click(screen.getByRole("button", { name: /save token/i }));
    expect(screen.getByTestId("voice-token-sheet")).toBeInTheDocument();
  });

  it("survives a localStorage that throws", () => {
    const getItem = vi
      .spyOn(Storage.prototype, "getItem")
      .mockImplementation(() => {
        throw new Error("denied");
      });
    const setItem = vi
      .spyOn(Storage.prototype, "setItem")
      .mockImplementation(() => {
        throw new Error("denied");
      });

    render(<VoiceCapture />);
    fireEvent.change(screen.getByLabelText(/display token/i), {
      target: { value: "abc" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save token/i }));
    // Token kept in component state even though persistence failed.
    expect(screen.getByTestId("voice-input")).toBeInTheDocument();

    getItem.mockRestore();
    setItem.mockRestore();
  });
});

describe("VoiceCapture — sending", () => {
  beforeEach(() => {
    localStorage.setItem("desk_token", "s3cret");
  });

  it("posts the transcript with the token header (happy path)", async () => {
    render(<VoiceCapture />);
    fireEvent.change(screen.getByTestId("voice-input"), {
      target: { value: "drzemka 5" },
    });
    fireEvent.click(screen.getByTestId("voice-send"));

    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe("/display/voice");
    expect(init.method).toBe("POST");
    expect(init.headers["X-Desk-Token"]).toBe("s3cret");
    const body = JSON.parse(init.body);
    expect(body.transcript).toBe("drzemka 5");
    expect(typeof body.captured_at_ms).toBe("number");
    // Textarea clears on success so the next note starts empty.
    await waitFor(() =>
      expect((screen.getByTestId("voice-input") as HTMLTextAreaElement).value).toBe(""),
    );
  });

  it("never posts a whitespace-only transcript (empty path)", () => {
    render(<VoiceCapture />);
    fireEvent.change(screen.getByTestId("voice-input"), {
      target: { value: "   " },
    });
    expect(screen.getByTestId("voice-send")).toBeDisabled();
    fireEvent.click(screen.getByTestId("voice-send"));
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("sends once when Send is double-clicked", async () => {
    let release: (v: unknown) => void = () => {};
    fetchMock.mockImplementation(
      () => new Promise((resolve) => { release = resolve; }),
    );

    render(<VoiceCapture />);
    fireEvent.change(screen.getByTestId("voice-input"), {
      target: { value: "notatka" },
    });
    const send = screen.getByTestId("voice-send");
    fireEvent.click(send);
    fireEvent.click(send);

    expect(fetchMock).toHaveBeenCalledTimes(1);
    await act(async () => {
      release({ ok: true, status: 200 });
    });
  });

  it("surfaces a rejected token (error path)", async () => {
    fetchMock.mockResolvedValue({ ok: false, status: 401 });
    render(<VoiceCapture />);
    fireEvent.change(screen.getByTestId("voice-input"), {
      target: { value: "notatka" },
    });
    fireEvent.click(screen.getByTestId("voice-send"));

    expect(await screen.findByTestId("voice-error")).toHaveTextContent(/wrong token/i);
    // The transcript is kept so the note is not lost.
    expect((screen.getByTestId("voice-input") as HTMLTextAreaElement).value).toBe("notatka");
  });

  it("surfaces a network failure (error path)", async () => {
    fetchMock.mockRejectedValue(new Error("offline"));
    render(<VoiceCapture />);
    fireEvent.change(screen.getByTestId("voice-input"), {
      target: { value: "notatka" },
    });
    fireEvent.click(screen.getByTestId("voice-send"));

    expect(await screen.findByTestId("voice-error")).toHaveTextContent(/no connection/i);
  });
});

describe("VoiceCapture — ack from the WS stream", () => {
  beforeEach(() => {
    localStorage.setItem("desk_token", "s3cret");
  });

  it("renders the intent and reply pushed through the ack bus", async () => {
    // `emitVoiceAck` is the same dispatch point useRemoteDesk's WS handler
    // calls for a `desk:voice-ack` frame (asserted in useRemoteDesk.test.ts).
    render(<VoiceCapture />);
    act(() => {
      emitVoiceAck({ transcript: "drzemka 5", intent: "Snooze(5)", reply: "Ok, 5 min." });
    });

    expect(await screen.findByTestId("voice-ack")).toHaveTextContent("Snooze(5)");
    expect(screen.getByTestId("voice-ack")).toHaveTextContent("Ok, 5 min.");
  });

  it("renders an ack with no AI reply", async () => {
    render(<VoiceCapture />);
    act(() => {
      emitVoiceAck({ transcript: "spacer", intent: "WalkStart", reply: null });
    });
    expect(await screen.findByTestId("voice-ack")).toHaveTextContent("WalkStart");
  });
});

describe("VoiceCapture — mic gating", () => {
  beforeEach(() => {
    localStorage.setItem("desk_token", "s3cret");
  });

  it("hides the mic button when SpeechRecognition is absent", () => {
    render(<VoiceCapture />);
    expect(screen.queryByTestId("voice-mic")).not.toBeInTheDocument();
  });

  it("hides the mic button when the microphone permission is denied", async () => {
    vi.stubGlobal("SpeechRecognition", class {});
    vi.stubGlobal("navigator", {
      ...navigator,
      permissions: { query: vi.fn().mockResolvedValue({ state: "denied" }) },
    });
    render(<VoiceCapture />);
    await waitFor(() => expect(screen.getByTestId("voice-send")).toBeInTheDocument());
    expect(screen.queryByTestId("voice-mic")).not.toBeInTheDocument();
  });

  it("shows the mic button when permissions.query throws", async () => {
    vi.stubGlobal("SpeechRecognition", class {});
    vi.stubGlobal("navigator", {
      ...navigator,
      permissions: {
        query: vi.fn(() => {
          throw new Error("unsupported name");
        }),
      },
    });
    render(<VoiceCapture />);
    expect(await screen.findByTestId("voice-mic")).toBeInTheDocument();
  });

  it("appends a recognised phrase to the textarea", async () => {
    let instance: FakeRecognition | null = null;
    class FakeRecognition {
      lang = "";
      interimResults = false;
      continuous = false;
      onresult: ((e: unknown) => void) | null = null;
      onerror: (() => void) | null = null;
      onend: (() => void) | null = null;
      constructor() {
        instance = this as unknown as FakeRecognition;
      }
      start() {}
      stop() {
        this.onend?.();
      }
    }
    vi.stubGlobal("SpeechRecognition", FakeRecognition);
    vi.stubGlobal("navigator", {
      ...navigator,
      permissions: { query: vi.fn().mockResolvedValue({ state: "granted" }) },
    });

    render(<VoiceCapture />);
    fireEvent.click(await screen.findByTestId("voice-mic"));
    act(() => {
      instance?.onresult?.({ results: [[{ transcript: "drzemka 10" }]] });
    });

    expect((screen.getByTestId("voice-input") as HTMLTextAreaElement).value).toBe(
      "drzemka 10",
    );
    // A second click stops listening and flips the label back.
    fireEvent.click(screen.getByTestId("voice-mic"));
    await waitFor(() =>
      expect(screen.getByTestId("voice-mic")).toHaveTextContent("Mic"),
    );
  });
});
