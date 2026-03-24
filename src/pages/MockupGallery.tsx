/**
 * MockupGallery.tsx — Dev-only page rendering all scenarios side by side.
 *
 * Accessible at /#/mockup in dev mode. Shows each scenario as a
 * mini popup card with its name, description, and the full widget.
 * Used for visual validation before implementing UI changes.
 */
import { useState } from "react";
import { ALL_SCENARIOS, PRIMARY_SCENARIOS, SECONDARY_SCENARIOS } from "@/test/scenarios";
import type { Scenario } from "@/test/scenarios";
import { OneBarWidget } from "@/widgets/OneBarWidget";

/** Single scenario card. */
function ScenarioCard({ scenario, selected }: { scenario: Scenario; selected: boolean }) {
  return (
    <div
      style={{
        border: selected ? "2px solid #DAA520" : "1px solid #333",
        borderRadius: 8,
        width: 380,
        background: "#1a1a2e",
        overflow: "hidden",
        flexShrink: 0,
      }}
    >
      <div style={{ padding: "8px 12px", background: "#0d0d1a", borderBottom: "1px solid #333" }}>
        <div style={{ color: "#ccc", fontSize: 13, fontWeight: 600 }}>
          {scenario.id}: {scenario.name}
        </div>
        <div style={{ color: "#888", fontSize: 11, marginTop: 2 }}>{scenario.description}</div>
        <div style={{ color: "#666", fontSize: 10, marginTop: 2, fontStyle: "italic" }}>
          {scenario.context}
        </div>
      </div>
      <div style={{ padding: 4 }}>
        <OneBarWidget {...scenario.props} />
      </div>
    </div>
  );
}

export default function MockupGallery() {
  const [filter, setFilter] = useState<"all" | "primary" | "secondary">("primary");
  const [selected, setSelected] = useState<string | null>(null);

  const scenarios =
    filter === "primary"
      ? PRIMARY_SCENARIOS
      : filter === "secondary"
        ? SECONDARY_SCENARIOS
        : ALL_SCENARIOS;

  return (
    <div style={{ background: "#0a0a15", minHeight: "100vh", padding: 20, color: "#ccc" }}>
      <h1 style={{ fontSize: 18, marginBottom: 8 }}>Popup Mockup Gallery</h1>
      <p style={{ fontSize: 12, color: "#888", marginBottom: 16 }}>
        Visual validation of all popup states. Click a card to select it for review.
      </p>

      <div style={{ display: "flex", gap: 8, marginBottom: 20 }}>
        {(["primary", "secondary", "all"] as const).map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            style={{
              padding: "4px 12px",
              background: filter === f ? "#DAA520" : "#222",
              color: filter === f ? "#000" : "#ccc",
              border: "none",
              borderRadius: 4,
              cursor: "pointer",
              fontSize: 12,
            }}
          >
            {f}
          </button>
        ))}
      </div>

      <div style={{ display: "flex", flexWrap: "wrap", gap: 16 }}>
        {scenarios.map((s) => (
          <div key={s.id} onClick={() => setSelected(s.id)} style={{ cursor: "pointer" }}>
            <ScenarioCard scenario={s} selected={selected === s.id} />
          </div>
        ))}
      </div>
    </div>
  );
}
