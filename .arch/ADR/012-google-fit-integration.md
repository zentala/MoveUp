# ADR 012: Google Fit Walking Steps Integration

- **Status**: accepted, partially superseded by
  [ADR 020](020-health-source-inlet.md) (2026-09-06)
- **Date**: 2026-05-19
- **Epic**: E000-maintenance (Google Fit walking-steps badge in OneBarWidget)
- **What ADR 020 replaced (E021, 2026-09-06)**: everything below still
  describes the Google Fit *client* — OAuth2 offline access, per-call token
  refresh, source auto-discovery, the `auth_revoked` / `transient` split — and
  all of it is still true. Two things are not:

  1. The **Deprecation-risk mitigation** under *Consequences* claimed a
     backend swap would be "local to ~3 files" behind `GoogleFitClient` and
     the `StepsView` IPC contract. It would not have been: the vendor's name
     was in the command names, the DTO names and the component name. The seam
     is now the `HealthSource` trait plus the documented push contract
     (`POST /display/health`), and Google Fit is one registered source among
     others. Its shutdown is a source that stops registering.
  2. The **Related** file list below is pre-E021. `StepsView` →
     `HealthView`; `commands_google_fit.rs` (`get_steps_today` /
     `refresh_steps_now`) is deleted in favour of `commands_health.rs`
     (`get_health_today` / `refresh_health_now`); `StepsWidget.tsx` →
     `HealthWidget.tsx`. `GoogleFitService` now implements `HealthSource`
     and also fetches heart rate.

  Google Fit itself is **not** being removed (E021-D6): while it is the only
  registered source, the widget carries a dated "Google Fit ends late 2026"
  hint.

- **Context**: The desk app's posture + screen-time model (ADR 011) tracks sitting / standing / walking-away from the local sensor + idle detector. But "walking away" is currently inferred only from "user is away from the keyboard and screen" — it has no signal of *actual* walking activity. Step counts from the user's existing health platform (phone, watch, fitness tracker) would close that loop and let the app eventually weigh nudges against real movement (vision item in `.plan/vision/2026-03-24-business-vision.md` — "Health API integration").

  Goal for this iteration: surface today's step count next to the existing KPIs in the OneBar popup. Future: drive nudge timing from real movement, weekly walking trends in the analyst dashboard.

- **Decision**: Integrate Google Fit via REST API + OAuth2 offline access. A long-lived refresh token (obtained by a one-time CLI helper) is stored in `.env` and exchanged for short-lived access tokens on every poll. Step counts are fetched from `users/me/dataset:aggregate` for the local day window. Rendered as a compact KPI badge inside the existing `KpiStrip` (no new layout surface).

  **Key technical choices:**
  1. **REST API over Health Connect** — Health Connect is on-device only (Android, no cloud), so cannot be read by a desktop Tauri app. Google Fit's REST API is the only Google source reachable from Windows.
  2. **`reqwest 0.11`** — already in the project's transitive dep tree via Tauri plugins; no new HTTP stack added.
  3. **Per-call token refresh, no token cache** — Fit's aggregate-for-today is cheap, polling cadence is 5 min, and a 1-hour access token cache would add nontrivial state for marginal benefit. Cost: 1 extra HTTP call per refresh (~50ms).
  4. **Compact KPI badge UI inside KpiStrip** — reuses the existing visual language for "today's stats"; no new section, no chart yet. Slotted via a `children` prop on `KpiStrip` so the badge participates in the same flex-wrap row.
  5. **Opt-in via env vars** — `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REFRESH_TOKEN` in `apps/desk/.env`. Absence is a calm "Connect Google Fit" UI, never an error.
  6. **Auto-discovery of data source** — Google Fit users have many step data sources (estimated_steps, merge_step_deltas, per-device raw). The client discovers available sources, ranks (`merge_step_deltas` > `estimated_steps` > derived > raw), and caches the choice. Env var `GOOGLE_FIT_STEPS_SOURCE` overrides discovery.
  7. **Error classification** — `auth_revoked` (invalid_grant / 401) vs `transient` (everything else). Frontend surfaces a "reconnect" CTA for the first, a red dot for the second.
  8. **OAuth consent flow via standalone CLI script** — not in-app. The full OAuth dance (browser, redirect, code exchange) is a one-time setup. Embedding it in the Tauri app would add hundreds of lines for a rarely-used flow. Script `apps/desk/scripts/google-fit-auth.cjs` runs the loopback flow with zero npm deps.

- **Alternatives considered and rejected:**
  - **Health Connect** — on-device API, no cloud read for a Windows app. Would require a companion Android app + sync server. Out of scope.
  - **Samsung Health API** — account-bound; user would need a Samsung account + tracker. Less universal than Google Fit which aggregates from many sources.
  - **Fitbit API** — separate user account, separate hardware. Considered as a future plugin but not the right "default health source" for this user.
  - **Apple HealthKit** — requires macOS / iOS. Desk is Windows-first.
  - **In-app OAuth consent flow** — too much surface area for a one-time action. CLI helper is simpler, more debuggable, and re-runnable.
  - **Token caching with expiry tracking** — adds state for marginal latency benefit. Revisit if poll cadence increases or if Google adds aggressive rate limiting.

- **Consequences:**
  - **Positive**: Real step data closes a long-standing gap in the activity model. Foundation for future "walking + screen breaks" correlation features.
  - **Operational**: User must run the OAuth helper script once. If refresh token is revoked (Google policy: ~6 months inactivity or manual revoke at https://myaccount.google.com/permissions), the frontend shows a "reconnect" CTA — user re-runs the script.
  - **Deprecation risk**: Google has signaled Fit API deprecation in favor of Health Connect (which has no cloud read). If/when the REST API is retired, we will need either (a) an Android companion app pushing to a private endpoint, or (b) switching to a different aggregator (Strava, Garmin Connect, Fitbit). Mitigation: integration is encapsulated behind `GoogleFitClient` + the `StepsView` IPC contract; swapping the backend is local to ~3 files.
  - **Privacy**: Credentials live in `apps/desk/.env` (gitignored). Tokens never enter logs or event payloads. Sanitized error strings exposed to the UI; full reqwest errors go to `log::warn!` only.
  - **Testing**: HTTP path covered by `wiremock`-backed tests (auth-revoked, 5xx, malformed JSON, happy path). Frontend covered by RTL with mocked `invoke`.

- **Related**:
  - Source files: `apps/desk/src-tauri/src/google_fit*.rs`, `apps/desk/src-tauri/src/commands_google_fit.rs`, `apps/desk/src/components/StepsWidget.tsx`.
  - Helper: `apps/desk/scripts/google-fit-auth.cjs`.
  - Docs: `apps/desk/CLAUDE.md` § Google Fit Integration.
  - Vision: `.plan/vision/2026-03-24-business-vision.md` (Health API integration — future state).
