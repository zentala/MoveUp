# ADR 026: Application-owned reminders in the existing main window

- Status: accepted
- Date: 2026-10-28
- Authority: operator request and owning PO application-reminders-and-bar-recurrence handoff
- Related: [ADR 010](010-notification-escalating-silence.md), [ADR 012](012-analyst-dashboard-separate-window.md), [ADR 015](015-pure-ergo-engine.md), [ADR 025](025-window-runtime-diagnostics.md)

## Context

Short Windows banners do not satisfy the requested readable, persistent reminder. Existing policy resolves the schedule and values, but its executors use OS notifications or a separate WinAPI alert. Main currently hides on focus loss. A frontend card alone cannot preserve a reminder while main is hidden or remounting.

## Decision

Keep the engine and CommunicationPolicy authoritative. Route their enabled ergonomic reminder intents through one small application-local adapter into the existing main window. Legacy toast, popup and both selections remain accepted profile inputs and resolve to one application reminder; none, Silent and disabled channels remain silent. Do not rewrite live profile JSON or send duplicate OS/WinAPI alerts. Unrelated operational errors and separately invoked low-level OS-channel probes retain their existing paths; the ordinary product Test button is explicitly covered below.

The adapter owns bounded current/pending presentation state, monotonic revisions and message identities. A targeted event announces changes; an IPC snapshot recovers state on initial mount/remount. The frontend reconciles revisions so late events/snapshots or acknowledgements cannot resurrect an obsolete message. No notification history store, new polling loop or framework is introduced. Restart uses the existing persisted policy acknowledgement/schedule, not replay of stale presentation state.

Dismiss and snooze address the actual message identity and reach existing policy once, including its current persistence/error handling. Policy context retires only messages whose reasons became obsolete, rather than dismissing all categories on any Standing sample. Frontend code never recomputes ergonomic thresholds. Values and categories come from existing authoritative facts/intent, never from a hardcoded forty-minute assumption.

Automatic presentation shows main without activation. Ordinary focus loss does not hide an active reminder. Manual tray activation still allows keyboard use. Settings and Share retain their state and take presentation priority until their flow exits; one pending reminder then becomes visible. Separate Analyst and remote display are not reminder hosts. No short timeout dismisses the message.

The existing product `trigger_test_notification` command and its ordinary Settings button preview the new main-window channel, explicitly labelled as a test, preserving one shared rate limit across categories. An optional validated `ReminderCategory` selects Info, Warning, Urgent or Success; absence defaults to Info. The small labelled selector beside Test changes only preview appearance, never ergonomic events. Every preview remains `is_test=true`; dismissal/snooze cannot mutate ergonomic counters or real policy acknowledgement/snooze. This is not a new endpoint. The earlier diagnostic-test exception refers only to separate low-level OS-channel probes; no further live OS probe is authorized here.

Normal policy-owned device-disconnection notices also use the main-window channel. Their typed reason originates at the policy decision, remains valid while disconnected and retires on reconnect. Missing reminder metadata must not silently route a normal policy notice back to OS or separate WinAPI delivery. Exceptional direct persistence/shutdown errors remain outside this channel change.

## Alternatives and consequences

- Reusing the separate WinAPI alert would duplicate layout/hosting and fail the requested same-window interaction.
- A new toast library or event-only React overlay would add dependencies or lose messages at hidden/remount boundaries.
- Reuse existing Tauri window/event/IPC, policy, logging, generated types and test infrastructure. Native no-focus show and focus-loss handling require actual Windows evidence, distinct from browser mocks.
- This supersedes only OS/separate-popup presentation in ADR 010 for ergonomic reminders. Its cooldown, explicit engagement and configurable silence remain unchanged. ADR 012's separate Analyst and ADR 025's observational diagnostics remain unchanged.
- The independent sitting-baseline correction carries credited load/configured limit through existing policy signals; it does not change credit/history, colors or native exposure recovery.

## Verification

Exercise policy-to-adapter delivery with no duplicate channels; disabled/Silent, late/duplicate IDs, remount/listen/snapshot races, protected Settings/Share, acknowledgement failure/retry and obsolescence. Inspect full main420x240 and constrained/enlarged text states using existing browser tests and independent images. Verify actual Windows automatic no-focus persistence and deliberate return to status after reviewed build activation. Keep source, synthetic and installed evidence separate.
