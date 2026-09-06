---
id: E021-T05
status: pending
---

# E021-T05: VoiceCapture component (phone display)

## Acceptance

`src/components/VoiceCapture.tsx` (+ `voice-capture.css`), rendered on
`/display` only (not the desktop popup); textarea + Send; mic button
gated on `SpeechRecognition` presence and a
`navigator.permissions.query({name:"microphone"})` result ≠ `denied`
(wrap in try/catch — the query throws on some browsers); token sheet
storing `desk_token` in `localStorage` (try/catch); `POST /display/voice`
with `X-Desk-Token`; renders the `VoiceAck` from the WS stream (subscribe
via a small `onVoiceAck` callback added to `useRemoteDesk`'s options —
keep `UseDeskResult` unchanged). Scenario `voiceCapture*` in
`src/test/scenarios-other.ts`; mockup gallery entry. Tests:
`VoiceCapture.test.tsx` (four paths + double-click guard). Verify:
`npx vitest run --config vite.config.ts src/components/VoiceCapture.test.tsx`.
