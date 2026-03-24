---
description: Show popup mockup gallery for visual validation of UI states
---

# Popup Mockup — Visual Validation

Show the popup widget rendered with predefined scenarios for visual review.

## Steps

1. Check if Vite dev server is running on port 1443:
   ```bash
   curl -s http://localhost:1443/ > /dev/null 2>&1 && echo "running" || echo "not running"
   ```

2. If NOT running, start it in the background:
   ```bash
   cd C:/code/zntl-tray/apps/desk && pnpm dev &
   ```
   Wait a few seconds for it to start.

3. Give the user ONLY the link:
   ```
   http://localhost:1443/#/mockup
   ```
   Say: "Mockup gallery is ready: http://localhost:1443/#/mockup"

4. The gallery shows all scenarios from `src/test/scenarios.ts`:
   - **Primary**: Fresh start, Sitting green/yellow/overtime, Standing, Away, Back from away
   - **Secondary**: Good day, Disconnected

5. Wait for user feedback on each scenario.

6. If user requests changes → update widget code → tell user to refresh page.

7. After approval, update `UX-FLOW.md`.

## Adding New Scenarios

To add a scenario, edit `src/test/scenarios.ts`:
- Define a new `Scenario` object with `id`, `name`, `description`, `context`, and `props`
- Add to `PRIMARY_SCENARIOS` or `SECONDARY_SCENARIOS` array
- Refresh mockup page to see it

## Scenarios are also used for tests

The same scenario data powers both mockups AND unit tests.
When you add a test, use scenario props: `render(<OneBarWidget {...S05_STANDING_MID.props} />)`
