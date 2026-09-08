---
id: E021-T09
status: done
updated: 2026-09-08
evidence: c8b3797
---
# E021-T09: Docs + ADRs (health source inlet, voice glance)
## Acceptance
.arch/ADR/020-health-source-inlet.md, .arch/ADR/021-voice-in-on-phone-watch-as-glance.md (Context/Decision/Alternatives/Consequences, Status: accepted), supersede note in ADR 012, .arch/ARCHITECTURE.md remote-server + health-inlet sections, .arch/UX-FLOW.md rows, CLAUDE.md "Google Fit Integration" -> "Health sources" (keys, push contract, voice, webhook, BYOK), CONTRIBUTING.md developer contract, docs/REMOTE_DISPLAY.md voice setup, PROJECT.xml, .plan/decisions.jsonl E021-D1..D6, .plan/BACKLOG.md entries: candidate E022 (Android Health Connect companion) and "create .plan/epics/INDEX.md". scripts/check-e021-t09-docs.mjs modelled on check-e020-t07-docs.mjs: grounds every doc claim in a symbol (trait HealthSource, fn require_token, enum Intent, struct WebhookNotifier, fn reply), counts checks, fails on zero. Verify: node scripts/check-e021-t09-docs.mjs.
