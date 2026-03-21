# T031 — Widget Architecture for Floating Window

**Priority:** P1
**Depends on:** T027 (session.rs split), T030 (floating window fixes)
**Blocks:** T032, T033
**Wave:** 5

## Goal

Replace the hardcoded floating window UI with a pluggable widget system.
Core app provides data; widgets handle presentation.

## Architecture

### WidgetProps interface (new: `src/types.ts`)

```typescript
/** Data provided by core to every widget. */
export interface WidgetProps {
  // Connection
  connected: boolean;
  port: string | null;

  // Current state
  state: DeskState | null;
  deskHeightCm: number;

  // Session timing
  currentSessionSecs: number;     // how long in current state
  limitSecs: number;              // sitting limit (e.g. 2400)
  limitRemaining: number;         // decreases when sitting, increases when standing/away
  limitRatio: number;             // 0.0–1.0+ (limitRemaining / limitSecs inverted)

  // Break
  breakSecs: number;              // current break duration (standing or away)
  breakResetThreshold: number;    // seconds needed for full reset (e.g. 900)
  breakResetProgress: number;     // 0.0–1.0 how close to full reset

  // Previous session
  previousSession: PreviousSession | null;

  // Today
  todaySessions: SessionEntry[];
  todayChanges: number;
  todayStandingSecs: number;
  todaySittingSecs: number;
  todayScore: number;             // from T028 (0 until implemented)

  // Error
  error: string | null;

  // Actions
  onOpenSettings: () => void;
}

export interface PreviousSession {
  state: DeskState;
  durationSecs: number;
  wasEffective: boolean;          // true if break was ≥ threshold for credit
}
```

### Widget component contract

```typescript
/** Every widget is a React FC that receives WidgetProps. */
export type DeskWidget = React.FC<WidgetProps>;

/** Widget registration entry. */
export interface WidgetRegistration {
  id: string;                     // e.g. "one-bar", "timeline-zen"
  name: string;                   // human-readable: "One Bar", "Timeline Zen"
  component: DeskWidget;
}
```

### Widget registry (`src/widgets/registry.ts`)

Simple map, no dynamic loading:

```typescript
import { OneBarWidget } from "./OneBarWidget";
import { TimelineZenWidget } from "./TimelineZenWidget";

export const WIDGET_REGISTRY: WidgetRegistration[] = [
  { id: "one-bar", name: "One Bar", component: OneBarWidget },
  { id: "timeline-zen", name: "Timeline Zen", component: TimelineZenWidget },
];

export const DEFAULT_WIDGET = "one-bar";
```

### Config change (Rust: `config.rs`)

Add to `AppConfig`:

```rust
#[serde(default = "default_active_widget")]
pub active_widget: String,

fn default_active_widget() -> String {
    "one-bar".to_string()
}
```

### Data provider (`src/hooks/useWidgetData.ts`)

New hook that wraps `useDesk` + `useTimer` + computed values:

```typescript
export function useWidgetData(): WidgetProps {
  const desk = useDesk();
  const liveSitting = useTimer(desk.sittingSeconds, desk.state === "Sitting");
  const liveBreak = useTimer(desk.breakSeconds, desk.state !== "Sitting" && desk.state !== null);
  const [todaySummary, setTodaySummary] = useState<TodaySummaryDto | null>(null);
  const [showSettings, setShowSettings] = useState(false);

  // Compute limitRemaining: starts at limitSecs, decreases while sitting
  // When standing/away: limitRemaining increases (break drains the limit used)
  const limitRemaining = desk.sessionLimitSecs - liveSitting;
  const limitRatio = desk.sessionLimitSecs > 0
    ? Math.min(liveSitting / desk.sessionLimitSecs, 1.5)
    : 0;

  // Previous session from todaySummary.sessions
  const previousSession = derivePreviousSession(todaySummary?.sessions ?? []);

  // Break reset progress
  const breakResetThreshold = 900; // 15 min — from config eventually
  const breakResetProgress = Math.min(liveBreak / breakResetThreshold, 1.0);

  return { ...computed values };
}
```

### App.tsx refactor

```tsx
export default function App() {
  const widgetData = useWidgetData();
  const [activeWidgetId] = useActiveWidget(); // reads from config
  const ActiveWidget = resolveWidget(activeWidgetId);

  if (widgetData.showSettings) {
    return <SettingsPanel onClose={...} />;
  }

  return (
    <main className="app">
      <ActiveWidget {...widgetData} />
    </main>
  );
}
```

## Files to create

| File | Purpose | Lines |
|------|---------|-------|
| `src/types.ts` | Add WidgetProps, PreviousSession, WidgetRegistration | +40 |
| `src/widgets/registry.ts` | Widget registry + resolver | ~30 |
| `src/hooks/useWidgetData.ts` | Compute derived props from useDesk | ~80 |
| `src/widgets/PlaceholderWidget.tsx` | Minimal widget that shows raw data (dev) | ~40 |

## Files to modify

| File | Change |
|------|--------|
| `src/App.tsx` | Replace hardcoded layout with widget system |
| `src-tauri/src/config.rs` | Add `active_widget: String` |
| `src/components/SettingsPanel.tsx` | Add widget picker dropdown |

## What moves OUT of App.tsx

- `ScreenProgressBar` — stays in App.tsx (overlay, not widget)
- `AppProgressBar` — moves into widget (widget decides if it wants it)
- `HeightRail` + `StateIndicator` + `SessionProgress` + `TodayStats` — absorbed by widgets
- Debug overlay info — stays in App.tsx (dev only)
- Error banner — provided via `widgetData.error`
- "stop" button — REMOVED (dev command, not user-facing)

## Tests

- `useWidgetData` hook: mock useDesk, verify computed limitRemaining, breakResetProgress, previousSession
- Widget registry: resolveWidget returns correct component, falls back to default
- Config: active_widget serialization roundtrip
- PlaceholderWidget: renders without crash with mock WidgetProps

## Acceptance criteria

- [ ] App.tsx uses widget system, no hardcoded session UI
- [ ] SettingsPanel has widget picker (dropdown)
- [ ] PlaceholderWidget shows all data from WidgetProps (dev verification)
- [ ] `active_widget` persisted in AppConfig
- [ ] All existing tests still pass
- [ ] No file > 250 lines
