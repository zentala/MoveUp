/**
 * VoiceCaptureGallery.tsx — dev-only gallery for the phone dictation panel.
 *
 * Reachable at `/#/mockup/voice` in dev. VoiceCapture takes no props: its
 * token comes from `localStorage` and its ack from the WS stream, and both
 * of those are global. So this gallery shows ONE scenario at a time —
 * side-by-side cards would fight over the same token and every card would
 * light up on the same ack.
 */
import { useState } from "react";
import { VoiceCapture } from "@/components/VoiceCapture";
import { emitVoiceAck } from "@/hooks/useRemoteDesk";
import { VOICE_CAPTURE_SCENARIOS, type VoiceCaptureScenario } from "@/test/scenarios";

const TOKEN_KEY = "desk_token";

/** Puts the scenario's token where VoiceCapture will look for it. */
function seedToken(s: VoiceCaptureScenario): VoiceCaptureScenario {
  if (s.token) localStorage.setItem(TOKEN_KEY, s.token);
  else localStorage.removeItem(TOKEN_KEY);
  return s;
}

export default function VoiceCaptureGallery() {
  // Seeded in the initializer so the token is in place before the panel's
  // first render reads it — an effect would run one render too late.
  const [scenario, setScenario] = useState<VoiceCaptureScenario>(() =>
    seedToken(VOICE_CAPTURE_SCENARIOS[0]),
  );
  // Bumped on every selection so VoiceCapture remounts and re-reads the token.
  const [mountKey, setMountKey] = useState(0);

  const select = (s: VoiceCaptureScenario) => {
    setScenario(seedToken(s));
    setMountKey((k) => k + 1);
  };

  return (
    <div style={{ background: "#0a0a15", minHeight: "100vh", padding: 20, color: "#ccc" }}>
      <h1 style={{ fontSize: 18, marginBottom: 8 }}>Voice Capture Mockups</h1>
      <p style={{ fontSize: 12, color: "#888", marginBottom: 16 }}>
        The dictation panel as it appears on the phone display. The mic button only
        shows when this browser exposes SpeechRecognition and the microphone
        permission is not denied.
      </p>

      <div style={{ display: "flex", gap: 8, marginBottom: 20 }}>
        {VOICE_CAPTURE_SCENARIOS.map((s) => (
          <button
            key={s.id}
            onClick={() => select(s)}
            style={{
              padding: "4px 12px",
              background: scenario.id === s.id ? "#DAA520" : "#222",
              color: scenario.id === s.id ? "#000" : "#ccc",
              border: "none",
              borderRadius: 4,
              cursor: "pointer",
              fontSize: 12,
            }}
          >
            {s.id}: {s.name}
          </button>
        ))}
      </div>

      <div style={{ border: "1px solid #333", borderRadius: 8, width: 340, background: "#1a1a2e", overflow: "hidden" }}>
        <div style={{ padding: "8px 12px", background: "#0d0d1a", borderBottom: "1px solid #333" }}>
          <div style={{ color: "#888", fontSize: 11 }}>{scenario.description}</div>
          <div style={{ color: "#666", fontSize: 10, marginTop: 2, fontStyle: "italic" }}>
            {scenario.context}
          </div>
        </div>
        <VoiceCapture key={mountKey} />
        {scenario.ack && (
          <button
            type="button"
            onClick={() => emitVoiceAck(scenario.ack!)}
            style={{
              width: "100%",
              padding: "6px 0",
              background: "#222",
              color: "#ccc",
              border: "none",
              borderTop: "1px solid #333",
              cursor: "pointer",
              fontSize: 11,
            }}
          >
            replay ack: {scenario.ack.intent}
          </button>
        )}
      </div>
    </div>
  );
}
