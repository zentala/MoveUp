---
id: E013-T01
title: Add reproducible unsigned Windows release baseline
status: in-review
priority: high
effort: medium
type: feature
dependencies: []
tags: [ci, release, tauri, windows]
epic: E013
branch: abandoned (superseded on main by 1484809)
commit: "ci(release): add reproducible unsigned Windows baseline"
group: E013-2026-08-28-signed-tauri-pm3-deployment
created: 2026-08-29
completed_at: null
---

# E013-T01: Add reproducible unsigned Windows release baseline

## Objective

Build the current application from immutable MoveUp and shared-UI revisions on
`windows-latest`, run its test gates, and preserve a short-lived unsigned
diagnostic installer plus its SHA-256 manifest.

## Acceptance criteria

- [x] Manual workflow accepts only full SHA revisions for both repositories.
- [x] Workflow installs locked dependencies, runs frontend and Rust tests, and
      creates an NSIS installer.
- [x] Uploaded artifact includes installer and manifest with source revision,
      filename, byte size, and SHA-256.
- [x] Build products and dependency stores are ignored by Git.

## Tests

- PowerShell manifest smoke test with a temporary `.exe` fixture.
- YAML and diff validation.

## Review gate

The workflow itself still needs one successful run on GitHub Actions. Local
dependency installation could not be completed because npm registry access is
blocked in this execution environment.

## Outcome 2026-09-05

The branch implementation was abandoned, not merged. Main carries an
independent, later workflow (`1484809`, 2026-09-04) that covers the same task
in 98 lines instead of 120.

The one thing the branch had and main does not is a second checkout of
`zentala/zntl-tray` at a pinned SHA, plus revision validation for both repos.
That is obsolete: `zntl-tray` is the app's former home and is referenced today
only by archived epic documents, so the release no longer builds against it.
Dropping it is correct, and the acceptance criterion about "both repositories"
above no longer applies.

The branch ref `feat/E013-T01-release-baseline` is kept so nothing is lost; its
worktree was removed.
