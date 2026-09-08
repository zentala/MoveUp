---
id: E022-T09
status: done
updated: 2026-09-08
evidence: 0034b08
---
# E022-T09: Phone pairing + controls UI
## Acceptance
src/remote/PairScreen.tsx (+ test), src/remote/RemoteControls.tsx (+ test), #/pair route in App.tsx, RemoteControls mounted in the remote layout only when capabilities.control; styles in src/remote/remote.css; scenarios for the mockup gallery in src/test/scenarios.ts (pair-empty, pair-deeplink, pair-error-locked, controls-pending). Verify: npx vitest run --config vite.config.ts src/remote/PairScreen src/remote/RemoteControls.
