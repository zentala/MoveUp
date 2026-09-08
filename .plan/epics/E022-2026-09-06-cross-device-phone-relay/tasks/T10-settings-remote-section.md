---
id: E022-T10
status: done
updated: 2026-09-08
evidence: 8cc0e4d
---
# E022-T10: Desktop Settings -> Remote section
## Acceptance
src/components/settings/RemoteSection.tsx (+ test, <=100 lines per component -- split PairedDevicesList.tsx, PairingCodeCard.tsx), qrcode dependency, wired into the existing settings panel tabs, scenarios relay-disabled, relay-online-2-viewers, relay-unentitled, relay-pairing-code-shown in src/test/scenarios.ts before the component (ux-design-flow.md). Verify: npx vitest run --config vite.config.ts src/components/settings/RemoteSection src/components/settings/PairedDevicesList src/components/settings/PairingCodeCard.
