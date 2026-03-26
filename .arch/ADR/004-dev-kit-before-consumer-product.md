# ADR 004: Launch as Dev Kit Before Consumer Product

- **Status**: accepted
- **Date**: 2026-03-24
- **Epic**: pre-epic (business strategy)
- **Context**: Launching a consumer electronics product in the EU requires full CE certification (EMC testing, documentation, potentially RED if radio). This costs 7-70k PLN and takes months. We need to validate demand before committing that investment.
- **Decision**: Phase 1 launches as a "Development Kit" / "Evaluation Board" — sold to developers and early adopters who want to participate in building the platform. Consumer product launch deferred to Phase 3 (12-24 months).
- **Alternatives**:
  - **Consumer product from day one**: Full CE, designed enclosure, retail packaging. Cost: 30-100k+ PLN upfront. Risk: if no one buys it, money is lost. Rejected: too much capital risk before demand validation.
  - **Software only (no hardware)**: Sell only the app, users source their own sensor. Zero hardware cost. But: massive friction (user must buy parts, wire them, flash firmware). Rejected: too high barrier for non-technical users, kills adoption.
  - **Crowdfund first, build later**: Kickstarter before building. But: no working product to demo, just renders. Low credibility. Rejected: we already HAVE a working product — leverage that.
- **Consequences**:
  - Dev kit positioning provides partial regulatory flexibility (not a consumer product)
  - Must maintain dev kit appearance: visible electronics, plexi mount, no retail packaging
  - Marketing targets developers, not general consumers
  - Price: 299-399 PLN (higher margin than consumer product, covers iteration costs)
  - USB cable NOT included (user provides own) — reduces liability, reinforces "kit" positioning
  - Need landing page, not retail distribution
  - First 50 units sold directly (no Amazon/Allegro)
  - Feedback from dev kit users shapes consumer product design
