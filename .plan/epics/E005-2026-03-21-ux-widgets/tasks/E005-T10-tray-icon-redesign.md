---
id: E005-T10
epic: E005
status: completed
created: 2026-03-21
completed: 2026-03-22
original_id: T016
title: T016 — Tray icon redesign: white base + colored dot
---
# T016 — Tray icon redesign: white base + colored dot

**Status:** open
**Priority:** P2
**Branch:** feat/T016-tray-icon

---

## Goal

Replace the current solid-colored-square tray icon with a white desk silhouette base + small colored dot.
Standing should show gold dot (not gray).

## New mapping
- Sitting < 60% -> green dot
- Sitting 60-85% -> amber dot
- Sitting > 85% -> red dot
- Standing -> gold dot `#DAA520`
- Walking/Away -> gray dot

---

## Acceptance Criteria

- [ ] Standing -> tray icon shows gold dot (not gray)
- [ ] Sitting progression -> green/amber/red dot
- [ ] Away/disconnected -> gray dot
- [ ] White desk silhouette visible
- [ ] `cargo test` passes
