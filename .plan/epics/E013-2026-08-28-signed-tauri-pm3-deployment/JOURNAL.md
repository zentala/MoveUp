# E013 Journal

## 2026-08-28 - Planning created

- Scope fixed to the current MoveUp Tauri application.
- Local Cargo release build is blocked by Smart App Control generated-binary
  enforcement; the planned build target is GitHub-hosted Windows CI.
- No code-signing identity has been selected or exposed to this repository.

## 2026-08-28 - Wave 1 release baseline

- Added a manually dispatched Windows release-baseline workflow. It requires a
  full commit SHA, checks out that revision, runs TypeScript and Rust gates,
  builds an NSIS installer, and retains the unsigned diagnostic artifact for
  seven days only.
- Initially the workflow exposed a stale dependency on sibling repository
  `zentala/zntl-tray`. This was corrected: `AutostartToggle` now lives in
  MoveUp and the release workflow needs no cross-repository checkout or token.
- Added a SHA-256 release manifest. No signing identity, deployment secret,
  PM3 configuration, or internal-domain registration was added; each remains
  gated on the required provider and infrastructure decisions.
- Verified the release-manifest script with a temporary artifact and PowerShell
  parser. Local Vitest startup remains blocked by Windows Application Control
  (`esbuild` spawn `EPERM`), reinforcing the GitHub-hosted build gate.

## 2026-08-29 - Standalone repository correction

- Removed the stale `@zntl/shared-ui` file dependency and moved the small
  autostart control into MoveUp's own component tree.
- The baseline release workflow now checks out only MoveUp; it has no
  `zntl-tray` checkout and no `ZNTL_TRAY_READ_TOKEN` requirement.
- Verified the standalone TypeScript project with `tsc --noEmit` and confirmed
  no cross-repository dependency reference remains in source, lockfile, docs,
  or workflow files.
