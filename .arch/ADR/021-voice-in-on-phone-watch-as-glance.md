# ADR 021: Voice goes in on the phone, the watch is a glance surface, and intents are events

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E021 (smartwatch integration + voice dictation) — tasks T05, T06, T07, T08
- **Related**: [ADR 001](001-remote-display-web-kiosk.md),
  [ADR 005](005-open-core-software-model.md),
  [ADR 010](010-notification-escalating-silence.md),
  [ADR 015](015-pure-ergo-engine.md),
  [ADR 020](020-health-source-inlet.md),
  [E021](../../.plan/epics/E021-2026-09-06-smartwatch-integration/PLAN.md)

## Context

The desk app can already tell the user to stand up. It has no way for the
user to say anything back, and the two places they would want to — "give me
five more minutes", "I'm going for a walk", "note this down" — are exactly
the moments when they are not at the keyboard.

Three constraints shaped what could be built:

- **The watch (Galaxy Watch 4) cannot host our code cheaply.** Running our
  own firmware on it means a Wear OS app, a build chain and a pairing story
  for one user. What the watch *does* do for free is mirror the phone's
  notifications.
- **`SpeechRecognition` needs a secure context in Chrome.** `/display` is
  served over plain `http://<LAN-IP>:3390` (ADR 001), so the Web Speech API
  is not reliably available there. It works in some browsers and kiosk
  configurations and not in others, which is worse than not having it: a
  feature that silently fails on the user's actual phone is a feature that
  cannot be trusted.
- **[ADR 015](015-pure-ergo-engine.md) forbids adapters from carrying
  policy.** The session engine is a pure core. A voice route is an adapter.
  If "I'm walking" reached in and set `DeskState::Walking`, the engine would
  have two sources of truth about where the user is — the sensor and a
  sentence — with no way to reconcile them.

## Decision

**The phone is the bridge; the watch is a glance surface; a recognised intent
is an event, never an engine input.**

1. **Input happens on the phone**, in `VoiceCapture.tsx` on `/display`. The
   primary path is a textarea the user dictates into with the **keyboard's
   own mic** (Gboard, Samsung Keyboard) — OS dictation, which works on any
   origin. The `SpeechRecognition` mic button is progressive enhancement:
   shown only when `window.SpeechRecognition` exists *and*
   `navigator.permissions.query({name:"microphone"})` does not report
   `denied` (wrapped in try/catch — the query throws on some browsers).
   Where it is unavailable the button is absent, never broken.
2. **Output reaches the watch as a phone notification.** `notify_webhook.rs`
   (`struct WebhookNotifier`) pushes `{title, message, priority, tags}` to an
   ntfy-shaped endpoint; Android mirrors that notification to the paired
   watch. No watch-side code exists, and none is planned.
3. **`POST /display/voice`** (`remote_routes_voice.rs`) takes
   `{transcript(1..2000), lang?, captured_at_ms}` behind the same
   `require_token` guard as the health inlet (ADR 020), capped at 4 KiB.
4. **`enum Intent`** (`voice_intent.rs`) — `Snooze(u16)`, `Note`,
   `WalkStart`, `WalkEnd` — is parsed **offline**, by an ordered table of
   Polish and English regexes. `WalkEnd` is tested before `WalkStart`
   because "wracam ze spaceru" contains a walk. Unrecognised text is always
   a `Note`; the parser has no failure mode that loses a transcript.
5. **The pipeline writes events, not state.** In order: an `EventLogger`
   `VOICE <intent> <transcript≤80>` line → a `voice_notes` row → the intent's
   side effect → an optional AI reply → the `desk:voice-ack` broadcast → the
   webhook push. The **only** side effect any intent has is `Snooze`, and it
   sets `CommunicationPolicy`'s *existing* snooze fields — the same ones the
   alert popup's snooze button already sets (ADR 010). `WalkStart`,
   `WalkEnd` and `Note` change nothing but the record. Nothing in
   `session_*.rs` is touched, called or imported by this path.
6. **The AI reply is BYOK and optional.** `voice_ai.rs` (`fn reply`) calls
   OpenRouter with the user's own `OPENROUTER_API_KEY`, an 8-second timeout
   and a fixed ergonomics-coaching system prompt capped at 60 words. No key
   means no reply — a missing key is the feature being off, never an error.
   Everything local (capture, parser, inlet, webhook) ships in the open app;
   hosted AI stays behind the Pro line ADR 005 draws.

Every leg of the pipeline is independently optional. No database, no event
logger, no AI key, no webhook — each degrades to a shorter acknowledgement,
and the request still returns `200` with the intent it parsed.

## Alternatives

- **Custom firmware / a Wear OS app on the Galaxy Watch 4.** The only way to
  get a real microphone button on the wrist. Rejected: a whole second build
  chain, a pairing story, and a Play Store or sideload path, for one user and
  one sentence. Notification mirroring gets the output half for free, and the
  input half is not worth the platform.
- **`SpeechRecognition` as the primary path, keyboard dictation as fallback.**
  The obvious ordering, and wrong here: the secure-context requirement means
  the primary path would be the one that fails on the deployment we actually
  have. Inverting it puts the reliable path first and makes the fragile one
  invisible when it is unavailable.
- **Serving `/display` over HTTPS to unlock `SpeechRecognition`.** Needs a
  certificate for a LAN IP — a self-signed one the phone must be taught to
  trust, or a real name and a proxy. That is a homelab project, not a desk-app
  feature, and it would change ADR 001's "open the URL and it works" contract.
- **Letting `WalkStart` set the session state.** The tempting one: the user
  said they are walking, so record a walk. Rejected under ADR 015 — the
  engine would then hold two contradictory sources of truth about position,
  and the reconciliation rules (what if the sensor disagrees? what ends a
  spoken walk?) are a feature with its own design, not a side effect. The
  transcript is recorded; a manual override is a future engine change with
  its own ADR.
- **Server-side speech-to-text (Whisper on the desktop).** Removes the
  browser API problem entirely and would accept raw audio. Rejected for this
  epic: it is a model download, a GPU question and an audio-upload endpoint,
  where OS dictation already produces text for free. It stays open — the
  route takes a transcript, so a local STT step in front of it changes
  nothing downstream.
- **Parsing intent with the LLM instead of regexes.** Rejected: intent
  parsing must work with no API key, offline, and deterministically enough to
  unit-test with a 12-phrase table. The LLM is used for the reply, where
  being wrong is cosmetic, not for the branch that decides whether an alert
  gets snoozed.

## Consequences

- **Positive.** The user can answer the app from across the room, in Polish
  or English, with no install: open `/display`, tap the keyboard mic, speak.
  The reply arrives on the wrist through the phone's own notification mirror.
- **Positive.** `voice_notes` is a new data source in the Analyst catalog,
  and the `VOICE` lines in `events.log` make every dictation auditable next
  to the state transitions around it.
- **Cost.** The intent table is a regex list, so it is exactly as good as the
  phrases someone thought of. `RULE_COUNT` is asserted in the tests, so a
  pattern that fails to compile cannot silently shrink the parser — but a
  phrase nobody anticipated becomes a `Note`, quietly and by design.
- **Cost.** A second write route on the LAN. It shares `require_token` with
  the health inlet, so the boundary is one function, not two.
- **Boundary.** The watch has no code and no state. Anything that needs the
  watch to *do* something rather than *show* something is out of scope of this
  decision and needs a new one.
- **Privacy.** Transcripts are stored locally in SQLite and written to the
  local event log. They leave the machine only when the user has configured a
  webhook (their own endpoint) or an OpenRouter key (their own account). No
  default sends anything anywhere.

## Related

- Source: `src-tauri/src/voice_intent.rs`, `remote_routes_voice.rs`,
  `db_voice_notes.rs`, `voice_ai.rs`, `notify_webhook.rs`,
  `src/components/VoiceCapture.tsx`.
- User docs: [`docs/REMOTE_DISPLAY.md`](../../docs/REMOTE_DISPLAY.md)
  § Voice Dictation.
- Developer contract: [`CONTRIBUTING.md`](../../CONTRIBUTING.md)
  § Voice dictation is an event path.
- Architecture: [`ARCHITECTURE.md`](ARCHITECTURE.md) § Voice Inlet.
- UX: [`UX-FLOW.md`](../UX-FLOW.md) § 12. Voice Dictation.
