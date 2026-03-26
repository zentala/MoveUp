# ADR 006: CE Self-Declaration (Not Notified Body Certification)

- **Status**: accepted
- **Date**: 2026-03-24
- **Epic**: pre-epic (certification strategy)
- **Context**: CE marking can be done via self-declaration (manufacturer signs Declaration of Conformity) or via notified body (external certification agency). For simple electronic products under EMC Directive, self-declaration is legally sufficient.
- **Decision**: Use self-declaration (DoC) for CE marking. We commission EMC testing at a certified lab, prepare technical documentation ourselves, and sign the Declaration of Conformity as the manufacturer. No notified body involvement.
- **Alternatives**:
  - **Notified body certification**: External agency reviews everything, issues certificate. More credible, but: costs 2-5x more, takes longer, unnecessary for EMC-only products. Rejected: overkill for our directive scope.
  - **No CE at all**: Sell only as components/parts. But: assembled functional device = product in EU law. Risk of market surveillance action. Rejected: legal risk too high even for dev kit.
- **Consequences**:
  - We bear full legal responsibility for compliance
  - Must maintain technical file: test reports, risk assessment, DoC document
  - EMC lab testing still required (5-15k PLN) — we just don't need a third party to "approve" the results
  - Must keep documentation updated if design changes
  - If we later add WiFi (RED directive), may need notified body involvement for radio testing — reassess at that point
  - Self-declaration is standard practice for simple electronics — most Arduino/Raspberry Pi products use this path
