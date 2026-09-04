# E013 implementation handoff

Waves and task files will be defined after plan review.

## Mental model

Implement only the current Tauri MoveUp application. Build on a GitHub-hosted
Windows runner, sign through an approved external signing identity, then run
the installed signed executable under PM3. The embedded remote display is the
browser surface; its health endpoint is `/display/api`.

Do not change Smart App Control settings, add a broad Cargo exception, or use
any legacy/archive application.
