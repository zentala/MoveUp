# T031 — Widget Architecture for Floating Window

**Priority:** P1
**Depends on:** T027 (session.rs split), T030 (floating window fixes)
**Blocks:** T032, T033
**Wave:** 5

## Goal

Replace the hardcoded floating window UI with a pluggable widget system.
Core app provides data; widgets handle presentation.

## CEO Review Decisions (2026-03-21)

1. **Extend useDesk** instead of new useWidgetData hook — no wrapper, add computed fields directly
2. **Shared `<SessionTimeline>`** component — both widgets import it, pass height/style props
3. **Edge guards in useDesk** — limitRatio=0 when limitSecs=0, previousSession=null when no history
4. **`limit_used_secs` computed in Rust** — added to SessionStateDto, single source of truth
5. **limitRemaining goes negative** when over limit — no clamping, no separate overtime field. Negative = overtime = points deducted. Widget clamps bar at 100% visually, shows negative number.
6. **Log active_widget** on startup and on switch for debuggability
7. **Wave 1 must also split** db.rs, serial.rs, commands.rs, overlay_tests.rs (all >250L)

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

  // Session timing — ALL derived from Rust-side limit_used_secs
  currentSessionSecs: number;     // how long in current state
  limitSecs: number;              // sitting limit (e.g. 2400)
  limitRemaining: number;         // limitSecs - limit_used_secs. GOES NEGATIVE when over limit.
  limitRatio: number;             // limit_used_secs / limitSecs. Can exceed 1.0.

  // Break
  breakSecs: number;              // current break duration (standing or away)
  breakResetThreshold: number;    // seconds needed for full reset (e.g. 900)
  breakResetProgress: number;     // 0.0–1.0 how close to full reset

  // Previous session
  previousSession: PreviousSession | null;  // null on first session of day

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

### Extend useDesk hook (NOT a new hook)

**CEO review decision:** No `useWidgetData`. Extend `useDesk` directly with computed fields.

Add to `useDesk` return:
```typescript
// NEW computed fields in useDesk:
limitUsedSecs: number;        // from Rust SessionStateDto.limit_used_secs
limitRemaining: number;       // limitSecs - limitUsedSecs (can be negative!)
limitRatio: number;           // limitUsedSecs / limitSecs (>1.0 when over limit, 0 when limitSecs=0)
breakResetProgress: number;   // min(breakSecs / breakResetThreshold, 1.0)
previousSession: PreviousSession | null; // derived from todaySummary.sessions
todaySessions: SessionEntry[];
todayChanges: number;
```

Edge guards:
- `limitSecs === 0` → `limitRatio = 0`, `limitRemaining = 0`
- `todaySummary === null` → `previousSession = null`, `todaySessions = []`
- `state === null` → widget shows "waiting for data" state

`limit_used_secs` comes from Rust (single source of truth for break credit logic).
TypeScript NEVER recomputes break credit — just reads the value.
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
| `src/widgets/PlaceholderWidget.tsx` | Minimal widget that shows raw data (dev) | ~40 |
| `src/widgets/shared/SessionTimeline.tsx` | Shared timeline component (blocks, hover, ghost lines) | ~80 |

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
