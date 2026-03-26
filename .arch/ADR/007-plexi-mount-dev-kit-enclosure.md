# ADR 007: Laser-Cut Plexi Mount as Dev Kit Enclosure

- **Status**: accepted
- **Date**: 2026-03-24
- **Epic**: pre-epic (hardware/business strategy)
- **Context**: Dev kit needs a physical mounting solution to attach sensor under desk. Options range from 3D-printed cases to injection molding to simple flat mounts. The choice affects cost, production speed, and importantly — whether the product looks like a "consumer device" or a "dev kit" (regulatory implications).
- **Decision**: Laser-cut semi-transparent acrylic (plexi) sheet with standoffs and screws. Components visible through the mount. Intentionally prototypical appearance.
- **Alternatives**:
  - **3D-printed enclosure**: More enclosed, looks more "product-like". But: slow per-unit production, inconsistent quality, could push toward consumer product classification. Rejected: wrong signal for dev kit positioning.
  - **Injection-molded case**: Professional, consumer-ready. But: tooling costs 15-30k PLN, minimum order 500-1000 units, months of lead time. Rejected: Phase 3, not Phase 1.
  - **No mount (bare modules)**: Cheapest. But: user has to figure out mounting themselves, looks unprofessional even for a dev kit. Rejected: too much friction.
  - **CNC aluminum case**: Premium feel, good EMC shielding. But: 50-200 PLN per unit, overkill for dev kit. Rejected: cost too high for validation phase.
- **Consequences**:
  - Cost per mount: ~10-30 PLN (laser cutting service, batch of 20-50)
  - Visible electronics reinforces "development kit" classification
  - No EMC shielding benefit (plexi is non-conductive) — may need shielding tape or copper foil if EMC fails
  - Easy to iterate design (just change the DXF/SVG file)
  - Mounting mechanism: adhesive tape or screw-in bracket under desk
  - Semi-transparent plexi looks clean while showing internals — good balance between "dev kit" and "not ugly"
