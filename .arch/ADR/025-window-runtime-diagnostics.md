# ADR 025: Application-local window runtime diagnostics

- **Status**: accepted
- **Date**: 2026-10-27
- **Related**: [ADR 012](012-analyst-dashboard-separate-window.md),
  [ADR 015](015-pure-ergo-engine.md), [ADR 024](024-application-lifecycle-readiness.md)

## Decision

Window diagnostics belong to the application adapter, outside the ergonomic
engine, history, persistence and backend readiness. Reuse the lifecycle run
identity and daily EventLogger. Preserve the separate main and Analyst windows.
Diagnostics never trigger reload, restart, focus, window creation or supervision.

Native callbacks submit small typed observations with a nonblocking bounded
queue (128 entries). One background writer maintains the most recent 64 event
groups and writes through the existing EventLogger. Repeated equal observations
retain their first and last receipt and an occurrence count. Queue overflow,
disconnection, stale results and write failures remain observable counters;
missing diagnostic evidence never implies a healthy interface.
Before any adapter observation, window presence is unknown and collection is
not_requested. Absence requires actual lookup or destruction evidence.

Window and navigation generations and probe IDs qualify asynchronous results.
Old or interrupted results cannot replace current observations. Collection has
distinct received, timeout, not_requested, not_present, unsupported, interrupted
and collection_failed states. Capability registration and log/report persistence
have their own states. UTC is evidence, receipt sequence orders events, and probe
deadlines use an injected monotonic clock in the native adapter.

The Windows adapter will own WebView2 and real suspend/resume registrations and
their cleanup. One physical script invocation per window remains in flight until
its callback completes, even after a logical timeout. A fixed read-only script
may return bounded boot, React, route, query-status and DOM geometry hints;
never HTML, text, input, storage, configuration or ergonomic data. DOM presence
and JavaScript responsiveness cannot establish GPU painting or a crash cause.

Reports separate observations from hypotheses and are capped at 256 KiB during
serialization. A fully written, synced sibling temporary file is published by an
exclusive hard link, never overwriting an existing report. Unsupported publication
and IO failures are errors, without a destructive or partial-write fallback.
Only the invocation's temporary file is removed. Existing daily-log retention
applies; there is no additional cleanup policy or HTTP API.

Read checkpoint status through its existing read-only boundary, without waiting
on session locks or copying raw error messages into diagnostics. Result-returning
EventLogger writes preserve existing log callers and retry semantics. Diagnostic
failure cannot change core readiness or stop session/sensor work.

## Implementation boundaries and verification

The initial core supplies types, a result-bearing sink, bounded state, report
storage and deterministic regressions. Module registration does not activate
callbacks, a worker, a tray action or a second application instance. Later native,
frontend and tray integration use these boundaries rather than another logger.

Test blocked writers and full queues, event coalescing, history bounds, stale
generations/probe IDs, every unavailable state, writer recovery, bounded reports
and exclusive publication using existing Rust tests and disposable files.
Source tests and builds do not establish installed-runtime behavior. Native
callbacks and report collection need separate isolated Windows evidence;
real workstation suspend/resume and production replacement remain separate.
