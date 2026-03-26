# ADR 002: Use ToF Sensor Module (VL53L1X) Instead of Laser Rangefinder

- **Status**: accepted
- **Date**: 2026-03-24
- **Epic**: pre-epic (business/hardware strategy)
- **Context**: The desk height sensor needs to measure distance from under-desk to floor (~70-120cm). Options range from ultrasonic sensors to laser rangefinders to Time-of-Flight (ToF) modules. The choice affects certification cost, safety classification, and development complexity.
- **Decision**: Use ST VL53L1X ToF sensor module (Grove breakout board). The module uses a VCSEL (940nm IR laser) already classified as **Class 1 (eye-safe)** per IEC 60825-1. We use the complete module without modifying optics, which means laser safety certification is handled by ST at module level.
- **Alternatives**:
  - **Laser rangefinder (e.g., Bosch-style)**: Higher power laser, longer range (10-50m) — overkill for 70-120cm. Requires extensive laser safety certification (EN 60825-1). Cost: 20k-100k+ PLN for certification. Rejected: unnecessary capability, massive certification burden.
  - **Ultrasonic sensor (e.g., HC-SR04)**: No laser concerns at all. But: lower accuracy (±1cm vs ±1mm), affected by temperature, wider beam angle (measures floor + nearby objects), slower response. Rejected: insufficient accuracy for reliable sitting/standing detection.
  - **IR distance sensor (e.g., Sharp GP2Y0A)**: Analog output, limited range, non-linear response. Rejected: poor accuracy at 70-120cm range.
- **Consequences**:
  - Laser documentation is paperwork only (~0-3k PLN), not a full safety certification
  - EMC remains the primary certification cost
  - Module is ~40-70 PLN per unit — acceptable for dev kit BOM
  - Must NOT modify optics (no custom lenses, no beam focusing) — would void Class 1 classification
  - Range limit ~2m is sufficient (desk heights 60-130cm)
  - Already working in current prototype (VL53L1X on Seeed XIAO ESP32-C3)
