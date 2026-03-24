# UX Design Flow — Mockups Before Code

## Mandatory flow for ANY UI change

1. **Define scenario** — add to `src/test/scenarios.ts` if not already there
2. **Show mockup** — run `/popup-mockup` skill, user sees live app with fake data
3. **User approves** — "ok" / "change X" → iterate mockup until approved
4. **Implement** — code the approved design
5. **Verify** — run mockup scenarios again, confirm match
6. **Update UX-FLOW.md** — document what changed

## Never skip steps 2-3

If you implement UI without showing a mockup first, you WILL misunderstand
the user's intent. This has happened repeatedly. The mockup step exists
to prevent wasted work.

## Auto-serve mockups

When showing mockups to the user:
1. Start `pnpm dev` in background if not already running (port 1443)
2. Give the user only the link: `http://localhost:1443/#/mockup`
3. User opens, reviews, gives feedback
4. You iterate code, user refreshes page
Never ask the user to start the server themselves.

## Scenario-driven development

All UI states are defined as typed scenarios in `src/test/scenarios.ts`.
These are reused for:
- Visual mockups (dev-only `/mockup` route)
- Unit tests (render with scenario data, assert elements)
- Integration tests (inject scenario, verify output)

One source of truth for "what the app looks like in situation X".
