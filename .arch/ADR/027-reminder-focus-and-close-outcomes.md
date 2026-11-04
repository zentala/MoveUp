# ADR 027: Non-activating reminders and truthful close outcomes

- Status: accepted
- Date: 2026-11-04
- Authority: explicit operator request through T3; sole MoveUp PO
- Related: [ADR 026](026-application-owned-reminders.md), [ADR 025](025-window-runtime-diagnostics.md), [ADR 012](012-analyst-dashboard-separate-window.md)

## Context

The operator reports a remaining popup and "The reminder was dismissed, but the popup could not be hidden. Retry closing." The frontend discards the hide rejection cause; guards, scheduling and native hide can fail differently. The actual failing branch is UNKNOWN. An empty log and the earlier native Test Information preview do not establish real acknowledgement/persistence behavior or a cause.

## Decision

Automatic reminder presentation must not take focus or input from ANY currently active application. A game is only an example: the foreground application must remain active and continue receiving its normal keyboard and pointer input before, during and after automatic popup presentation or update. Automatic show/update must not focus or activate MoveUp, simulate engagement, or manipulate another application's windows. Reuse the existing application presentation adapter; preserve engine and policy ownership.

A deliberate click on MoveUp or a manual tray command is a separate engagement boundary and may activate the chosen MoveUp control. Automatic delivery cannot manufacture that consent. Do not forcibly restore another application's focus after deliberate interaction. Preserve Settings/Share protection and the separate Analyst window.

Exclusive fullscreen may prevent another window from being composited visibly. Do not promise visibility there or defeat that limitation by focus stealing, changing display mode, injecting overlays or repeatedly forcing topmost. Record visibility, foreground and input observations separately; borderless/windowed evidence does not prove exclusive-fullscreen behavior.

Real-reminder X acknowledges and persists the intended message once, then hides the owned main window. Capture the actual hide result with matching message/session/client/revision context and distinguish obsolete guard, scheduling and native failures. Preserve protections against hiding a newer reminder or protected view. An obsolete close is not automatically a native-hide failure.

The normal successful path requires one X without a retry ritual or leftover error strip. Do not cosmetically suppress a real unresolved failure or infer native success from resolved IPC alone. Genuine failures retain truthful bounded evidence and usable recovery; hide retry cannot repeat policy/persistence effects. Use existing bounded event/runtime diagnostics under ADR025, without a new collector, journal framework, reload or restart policy.

## Alternatives and consequences

Foreground activation would interrupt the active application and violates the requirement. Removing identity guards or merely hiding error text would conceal failures. This decision clarifies ADR026; scheduling, silence, test-preview isolation and protected flows remain unchanged. No history/credit rewrite, new supervisor or foreign-window control is authorized.

## Verification

Obtain the actual failing hide result before declaring a cause. Reproduce the confirmed mechanism and add discriminating regressions, independent source review and the existing check. Build the final relevant artifact and use the existing protected backup/rollback and sole PM3 installation route. On Windows separately verify real-reminder X/ACK/persistence/visibility and automatic presentation over representative ordinary applications. Record the foreground identity and actual keyboard receipt before, during and after automatic presentation, before any deliberate popup click. Use disposable non-sensitive targets and capture only necessary owned-window evidence. The contract is universal; representative checks do not prove every application was tested. A game or chosen fullscreen mode is not an acceptance prerequisite. Missing actual automatic presentation or input evidence remains NOT_CHECKED; synthetic tests and manually engaged preview cannot replace it. Exclusive-fullscreen visibility limitations never authorize focus stealing.

## Operator clarification — 2026-11-05

The operator explicitly clarified through T3 that this requirement covers every active program, not a particular game. The earlier PO question about a game and its mode, and the acceptance dependency created by that question, are withdrawn. Deliberate engagement remains a separate boundary; this clarification does not authorize forced focus, foreign-window manipulation or replay of refused operations.
