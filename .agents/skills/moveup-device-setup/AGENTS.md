# Device setup skill authors

This is the portable, repository-owned source of `moveup-device-setup`.
Read the root `AGENTS.md`, `firmware/AGENTS.md` and `firmware/README.md` before
editing it. Its purpose is to help a beginner move from purchased modules to a
verified USB desk sensor; manual UF2 flashing is current, app flashing is future.

Keep the actual wiring and serial protocol in the firmware README. The skill
selects the next stage and tells an agent how to verify it. Preserve the distinction
between a read-only program check, an authorized flash and a physical sensor test.
Do not bake a developer's port, paths, USB serial or machine services into it.

Validate instructions against the actual sketch and supported Arduino APIs.
For behavior changes, use a fresh-session trial and the real device when available;
source review does not establish novice usability or successful sensor assembly.
Keep trial results and future decisions in the owning project plan, not this
portable skill. Do not modify an installed global skill while working here.
