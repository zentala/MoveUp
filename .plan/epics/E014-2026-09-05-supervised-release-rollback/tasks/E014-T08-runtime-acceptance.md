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

Prove the actual daemon path handles a broken newest candidate, two 60-second
misses, fallback, persistent demotion, fixed-artifact retry and marker promotion.
Prove foreign healthy stand-down over at least two complete poll intervals,
alternate-port development suspension, acknowledged Quit, daemon replacement
within the same login and suspension expiry under a controlled login identity.
Retain timestamps, process identities, relevant logs, source revisions and exits.
Fake-clock unit tests supplement, not replace, the real runtime evidence.

Verify diagnostic logs from real isolated lifecycle fixtures: ready, graceful
exit, early failure/panic and forced termination without invented crash cause.
Check the Windows GUI/tray behavior using an isolated fixture before relying on
the supervisor launch flags. A runtime or visual check that was not performed
remains NOT_CHECKED. Record failures even after a later successful retry.

Independent source review and repository checks must pass before staging a live
cutover. Live PM3 release holds remain binding; report their exact owner/evidence.
No reboot, logoff, live daemon replacement or production failure injection here.
