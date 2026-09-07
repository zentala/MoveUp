/**
 * VoiceCapture.tsx — dictation panel for the phone display (`/display`).
 *
 * Rendered only in remote-display (browser) mode; the desktop popup has a
 * keyboard and does not need it. Keyboard dictation into the textarea is the
 * primary path (E021-D5); the mic button is progressive enhancement and only
 * appears when the browser exposes `SpeechRecognition` and the microphone
 * permission is not already denied.
 *
 * Notes are posted to `POST /display/voice` with the shared `X-Desk-Token`
 * header. The acknowledgement comes back asynchronously on the WebSocket
 * stream as `desk:voice-ack`, not in the POST response.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { subscribeVoiceAck, type VoiceAck } from "@/hooks/useRemoteDesk";
import "./voice-capture.css";

/** localStorage key holding the shared remote-display token. */
const TOKEN_KEY = "desk_token";

/** Minimal shape of a `SpeechRecognition` instance — we use four members. */
interface SpeechRecognitionLike {
  lang: string;
  interimResults: boolean;
  continuous: boolean;
  start(): void;
  stop(): void;
  onresult: ((e: { results: ArrayLike<ArrayLike<{ transcript: string }>> }) => void) | null;
  onerror: (() => void) | null;
  onend: (() => void) | null;
}

/** Reads the token, tolerating a `localStorage` that throws (private mode). */
function readToken(): string {
  try {
    return localStorage.getItem(TOKEN_KEY) ?? "";
  } catch {
    return "";
  }
}

/** Persists the token, tolerating a `localStorage` that throws. */
function writeToken(value: string): void {
  try {
    localStorage.setItem(TOKEN_KEY, value);
  } catch {
    // Non-fatal: the token still lives in component state for this session.
  }
}

/** The browser's `SpeechRecognition` constructor, or null when unsupported. */
function speechRecognitionCtor(): (new () => SpeechRecognitionLike) | null {
  const w = window as unknown as Record<string, unknown>;
  const ctor = w.SpeechRecognition ?? w.webkitSpeechRecognition;
  return (ctor as (new () => SpeechRecognitionLike) | undefined) ?? null;
}

/**
 * Whether the mic button may be shown: the API exists AND the microphone
 * permission is not already `denied`. `permissions.query` throws on some
 * browsers (and rejects for unknown names on others) — a failed query means
 * "unknown", which we treat as available rather than hiding the button.
 */
function useMicAvailable(): boolean {
  const [available, setAvailable] = useState(false);

  useEffect(() => {
    if (!speechRecognitionCtor()) return;
    let cancelled = false;
    (async () => {
      let denied: boolean;
      try {
        const status = await navigator.permissions.query({
          name: "microphone" as PermissionName,
        });
        denied = status.state === "denied";
      } catch {
        denied = false;
      }
      if (!cancelled) setAvailable(!denied);
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return available;
}

/** Dictation panel: token sheet, textarea, optional mic, ack from the stream. */
export function VoiceCapture() {
  const [token, setToken] = useState(readToken);
  const [tokenDraft, setTokenDraft] = useState("");
  const [transcript, setTranscript] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [ack, setAck] = useState<VoiceAck | null>(null);
  const [listening, setListening] = useState(false);
  const recognition = useRef<SpeechRecognitionLike | null>(null);
  const micAvailable = useMicAvailable();

  useEffect(() => subscribeVoiceAck(setAck), []);

  const send = useCallback(async () => {
    const text = transcript.trim();
    if (sending || !text || !token) return;
    setSending(true);
    setError(null);
    try {
      const res = await fetch("/display/voice", {
        method: "POST",
        headers: { "Content-Type": "application/json", "X-Desk-Token": token },
        body: JSON.stringify({
          transcript: text,
          lang: navigator.language,
          captured_at_ms: Date.now(),
        }),
      });
      if (!res.ok) {
        setError(res.status === 401 ? "Wrong token" : `Send failed (${res.status})`);
        return;
      }
      setTranscript("");
    } catch {
      setError("Send failed — no connection");
    } finally {
      setSending(false);
    }
  }, [transcript, sending, token]);

  const toggleMic = useCallback(() => {
    if (listening) {
      recognition.current?.stop();
      return;
    }
    const Ctor = speechRecognitionCtor();
    if (!Ctor) return;
    const rec = new Ctor();
    rec.lang = navigator.language;
    rec.interimResults = false;
    rec.continuous = false;
    rec.onresult = (e) => {
      const said = e.results[0]?.[0]?.transcript ?? "";
      setTranscript((prev) => (prev ? `${prev} ${said}` : said));
    };
    rec.onerror = () => setListening(false);
    rec.onend = () => setListening(false);
    recognition.current = rec;
    rec.start();
    setListening(true);
  }, [listening]);

  if (!token) {
    return (
      <section className="voice" data-testid="voice-capture">
        <form
          className="voice__token-sheet"
          data-testid="voice-token-sheet"
          onSubmit={(e) => {
            e.preventDefault();
            const value = tokenDraft.trim();
            if (!value) return;
            writeToken(value);
            setToken(value);
          }}
        >
          <label className="voice__label" htmlFor="voice-token">
            Display token
          </label>
          <input
            id="voice-token"
            className="voice__token-input"
            type="password"
            value={tokenDraft}
            onChange={(e) => setTokenDraft(e.target.value)}
            placeholder="DESK_REMOTE_TOKEN"
          />
          <button className="voice__btn" type="submit">
            Save token
          </button>
        </form>
      </section>
    );
  }

  return (
    <section className="voice" data-testid="voice-capture">
      <textarea
        className="voice__input"
        data-testid="voice-input"
        aria-label="Voice note"
        rows={2}
        value={transcript}
        placeholder="Say or type a note…"
        onChange={(e) => setTranscript(e.target.value)}
      />
      <div className="voice__row">
        {micAvailable && (
          <button
            className={`voice__btn voice__btn--mic${listening ? " is-listening" : ""}`}
            data-testid="voice-mic"
            type="button"
            aria-label={listening ? "Stop dictation" : "Start dictation"}
            onClick={toggleMic}
          >
            {listening ? "Stop" : "Mic"}
          </button>
        )}
        <button
          className="voice__btn voice__btn--send"
          data-testid="voice-send"
          type="button"
          disabled={sending || !transcript.trim()}
          onClick={send}
        >
          {sending ? "Sending…" : "Send"}
        </button>
      </div>
      {error && (
        <p className="voice__error" data-testid="voice-error">
          {error}
        </p>
      )}
      {ack && (
        <p className="voice__ack" data-testid="voice-ack">
          <span className="voice__ack-intent">{ack.intent}</span>
          {ack.reply ? ` — ${ack.reply}` : ""}
        </p>
      )}
    </section>
  );
}

export default VoiceCapture;
