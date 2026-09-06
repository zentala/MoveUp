---
id: E021-T02
status: pending
---

# E021-T02: Health source trait + Google Fit adapter

## Acceptance

`health_source.rs` (trait `HealthSource { fn id(&self) -> &str; async fn view(&self) -> HealthView; async fn refresh(&self) -> HealthView }`,
`HealthAggregator` merging by `fetched_at_ms`, error passthrough),
`health_models.rs` (`HealthSnapshot`, `HealthView`, `HealthErrorKind`
— reuse `ErrorKind`'s two variants), `GoogleFitService` implements the
trait, HR via `com.google.heart_rate.bpm` aggregate (`Option`, skip
when T01 said no), `commands_health.rs` (`get_health_today`,
`refresh_health_now`), delete `commands_google_fit.rs`, both invoke
lists in `lib.rs`. Tests: `health_source_tests.rs`, HR case in
`google_fit_http_tests.rs`, four paths per PLAN. Verify:
`cargo test --manifest-path src-tauri/Cargo.toml --lib -- health_source google_fit`.
