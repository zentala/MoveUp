---
formatVersion: 1
id: E014-T08
title: Verify supervised recovery through the actual runtime
epic: E014
status: pending
points: 5
importance: High
dependencies: [E014-T07]
---

# Runtime acceptance

Execute the acceptance matrix in ../RESUME.md against reviewed integrated source.
Use a separate PM3 home, ports, synthetic executables/releases and app data. Keep
the real user's application, profile, database and active PM3 daemon untouched.

Apply every public fixture environment key to each CLI/daemon child, clearing
unrelated inherited PM3_TEST overrides first. PM3_HOME alone does not isolate
the Windows named pipe. Accepted PM3's daemon/test-overrides.ts also strips test
keys from its own environment after reading them, so inheritance through the
daemon is insufficient for a supervised application that calls PM3 back. Carry
the fixture IPC/run/pid overrides explicitly through the test service's env
configuration (or another verified public fixture mechanism). Assert their
actual presence in that child before exercising its Quit/dev adapter. Keep
test overrides out of production configuration. Use the supported no-op domain
adapter for fixtures so their registration cannot touch the live domain registry.

The current app resolves data through Tauri's `app.path().app_data_dir()` at
multiple call sites; no application data-directory override was found in the
current runtime. Do not assume setting APPDATA alone isolates it. Establish a
test-specific application identity or another verified fixture mechanism, and
prove the resolved data directory, singleton identity and disabled autostart
before launching a GUI fixture. Keep sensor access and outbound integration
isolated as well. Record fixture/build differences; a fixture cannot replace
the final installed-release desktop/login check.

Prove the actual daemon path handles a broken newest candidate, two 60-second
misses, fallback, persistent demotion, fixed-artifact retry and marker promotion.
Prove foreign healthy stand-down over at least two complete poll intervals,
alternate-port development suspension, acknowledged Quit, daemon replacement
within the same login and suspension expiry under a controlled login identity.
Retain timestamps, process identities, relevant logs, source revisions and exits.
Fake-clock unit tests supplement, not replace, the real runtime evidence.

Cover both process exit and a still-alive process whose typed readiness stops
passing after a successful promotion probe. In both cases measure the full
replacement window from the last positive observation; a fixed poll schedule
must not shorten two 60-second misses to 90 seconds. Preserve the demotion of
an unchanged artifact when its candidate temporarily disappears and returns.
Include unavailable login identity and persistence-failure outcomes: unknown
state cannot authorize Quit or silently resume a suspended service, and failed
durable writes must be visible rather than reported as successful persistence.

Verify diagnostic logs from real isolated lifecycle fixtures: ready, graceful
exit, early failure/panic and forced termination without invented crash cause.
Check the Windows GUI/tray behavior using an isolated fixture before relying on
the supervisor launch flags. A runtime or visual check that was not performed
remains NOT_CHECKED. Record failures even after a later successful retry.

Independent source review and repository checks must pass before staging a live
cutover. Live PM3 release holds remain binding; report their exact owner/evidence.
No reboot, logoff, live daemon replacement or production failure injection here.
