---
id: E022-T04
status: pending
---
# E022-T04: Relay command routing
## Acceptance
command from a viewer: validate name/args with the shared zod schema, attach viewer_id, forward to the desk socket; desk_offline immediate result when none; route command_result by command_id to the originating viewer only (keep a bounded Map<command_id, viewer_tag> with TTL 30s); 10/min/viewer. Verify: pnpm --dir relay exec vitest run test/commands.
