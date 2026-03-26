# ADR 003: USB-Only Communication (No WiFi/BLE) in Phase 1

- **Status**: accepted
- **Date**: 2026-03-24
- **Epic**: pre-epic (business/hardware strategy)
- **Context**: ESP32-C3 supports WiFi and BLE natively. Activating radio features would enable wireless operation but triggers the EU Radio Equipment Directive (RED 2014/53/EU), requiring RF + EMC testing at 15-50k PLN. Phase 1 goal is to validate the product with minimal certification cost.
- **Decision**: Phase 1 uses USB serial communication only. WiFi and BLE radios on ESP32-C3 are NOT activated in firmware. The device is classified as a non-radio product, subject only to EMC Directive (not RED).
- **Alternatives**:
  - **WiFi from day one**: Enables standalone phone mode immediately. But: adds 15-50k PLN certification cost, delays launch, increases firmware complexity. Rejected: cost/risk too high for validation phase.
  - **BLE only**: Lower power, simpler protocol. But: still triggers RED. Marginally cheaper than WiFi testing but same directive. Rejected: same regulatory burden as WiFi.
  - **Different MCU without radio (RP2040, ATmega328)**: Eliminates even the possibility of RED questions. But: requires porting working firmware, buying new dev boards, re-testing. Rejected for now: ESP32-C3 works, radio is simply not activated. Revisit if regulators challenge this.
- **Consequences**:
  - Certification cost reduced from ~20-70k PLN to ~5-15k PLN
  - Device requires USB connection to computer (no standalone mode)
  - Phone-as-display (E009) works via PC as intermediary (web kiosk over LAN)
  - Standalone phone mode (Phase 3) deferred until consumer product stage
  - Wireless capability added later when funded by Kickstarter/grants
  - If using ESP32-C3 without radio activation raises regulatory questions, consider MCU swap to RP2040 (no radio hardware at all)
