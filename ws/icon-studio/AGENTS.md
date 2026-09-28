# AGENTS.md — MoveUp icon studio

## What this is

This workspace develops editable SVG icons from the three June 8 reference boards. Its output is a reviewable library, not approved application assets.

## Scope

- Work in `ws/icon-studio/`. The local plan is `.plan/`; start with `.plan/epics/INDEX.md` and the E026 `PLAN.md` and `HANDOFF.md`.
- Preserve the source PNGs in `references/` unchanged. Keep the existing approximate icon families distinct from reference-derived vectors.
- Do not change MoveUp runtime, tray icons, or website assets during E026. Integration requires a later selection and plan.
- Use `pnpm icons:generate` from the MoveUp repository root to rebuild SVGs. Review generated icons against their source crops at 48, 24, and 16 px.
- Every proposed icon must have a source identity, crop, editable SVG, and explicit human review verdict. A successful batch with zero icons is a failure.

## Ownership

This workspace's `.plan/` owns icon-library tasks and evidence. The MoveUp root `.plan/epics/INDEX.md` links here for discovery; it does not duplicate task state.
