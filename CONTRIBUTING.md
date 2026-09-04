# Contributing to MoveUp

This guide is for developers and coding agents. It is not required to evaluate or
use a future released MoveUp installer.

## Development model

MoveUp is one Tauri desktop process: the tray app owns the sensor, local SQLite
data and the embedded browser dashboard. Its remote dashboard is served by that
same process over HTTP and WebSocket; it is not a separate product server.

- **Ordinary development:** run `pnpm tauri:dev` (or its `:live`, `:mock` or
  `:demo` variant). `scripts/tauri-dev.ps1` avoids colliding with an existing
  development instance.
- **Local always-on use:** PM3 may supervise a release MoveUp binary with
  `--minimized`, then `idomains` may expose its chosen dashboard port as a local
  `*.internal` name. PM3 is a workstation convenience, not a runtime dependency
  of MoveUp and not something shipped to end users.
- **Distribution:** ship a signed Windows installer containing the release
  binary. Do not ship Rust, Cargo, PM3 or a developer helper as part of the
  user-facing product.

## Prerequisites

- Windows with the Rust MSVC toolchain and Node.js/pnpm;
- optional: the VL53L1X desk controller for live sensor work.

Install dependencies and run checks from the repository root:

```powershell
pnpm install
pnpm test:all
pnpm tauri:build
```

## Windows Application Control

Cargo generates small unsigned executable build scripts for dependencies during
a Rust build. Some Windows Application Control (WDAC/App Control) policies block
those scripts with `os error 4551`. That is a developer-machine policy issue,
not something a MoveUp user should encounter.

Do not disable App Control globally and do not add a broad antivirus exclusion.
If your managed development machine blocks Cargo, request a narrowly scoped
developer policy from its administrator for the approved Rust toolchain and this
repository's generated build artifacts. Record the policy owner and review it when
the build toolchain changes. A signed release installer is the correct answer for
end-user machines; it is not a substitute for a controlled developer policy.

## Release baseline (CI)

The `Release baseline` workflow is manual and accepts only a full 40-character
commit SHA. It checks out that exact revision, runs the frontend and Rust test
gates, builds the NSIS installer, and publishes a seven-day unsigned diagnostic
artifact with `manifest.json`. The manifest records the source revision,
installer filename, byte size, and SHA-256 hash.

This baseline is not deployable output. Do not install it on a Smart App Control
machine or use it as a signing workaround. The next release stage must use an
approved managed signing provider, verify Authenticode trust and timestamping,
and only then publish a signed installer.

## Before planning changes

Load the global `how-we-build` skill before writing a plan or making a design
decision. It contains the ecosystem conventions, including the distinction between
this file and README.
