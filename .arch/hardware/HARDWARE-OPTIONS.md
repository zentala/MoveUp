# Hardware Design Options & Component Research

**Date**: 2026-03-25
**Status**: Research (decisions not finalized)
**Source**: ChatGPT hardware consultation + zentala's requirements

---

## 1. Current Prototype

| Component | Part | Notes |
|-----------|------|-------|
| MCU | Seeed XIAO ESP32-C3 | Has WiFi/BLE but we don't use it (ADR 003) |
| Sensor | Grove VL53L1X v2 | ToF, Class 1 laser, I2C |
| Connection | USB serial (COM3) | 115200 baud |
| Mount | None (breadboard) | Needs productization |

**Problem with current setup:**
- ESP32-C3 has WiFi/BLE hardware we don't use → regulatory ambiguity
- No mounting solution → not shippable as dev kit
- No physical enclosure or carrier

---

## 2. MCU Options (Without WiFi/BLE)

If we want to eliminate radio hardware entirely (cleaner for CE/EMC):

### Recommended: Seeed XIAO RP2040

| Spec | Value |
|------|-------|
| Size | ~21×17 mm (ultra small) |
| CPU | RP2040 dual-core ARM Cortex-M0+ |
| USB | USB-C (native) |
| Price | ~20-30 PLN (~4-5 EUR) |
| WiFi/BLE | **None** |
| Mounting holes | **None** (standard for this size) |
| Ecosystem | Arduino, MicroPython, PlatformIO |
| I2C | Yes (for VL53L1X) |

**Why this over ESP32-C3:**
- Zero radio hardware → zero RED ambiguity
- Cheaper (~20 PLN vs ~30 PLN)
- USB-C native
- Dual-core (overkill but future-proof)
- Same XIAO form factor → minimal firmware changes

**Why NOT this:**
- No mounting holes (solved by carrier PCB — see below)
- Firmware needs porting from ESP32 Arduino to RP2040 Arduino (moderate effort)
- Loses WiFi option for future standalone mode (but that's Phase 3 anyway)

### Alternative: Keep ESP32-C3

- Already working, firmware done
- WiFi/BLE present but disabled in firmware
- Slightly more expensive
- Regulatory gray area (radio hardware exists even if unused)
- **Decision: defer MCU change until production batch. Prototype stays on ESP32-C3.**

### Other options considered (from ChatGPT research):

| MCU | Price | USB-C | Mounting | Notes |
|-----|-------|-------|----------|-------|
| XIAO RP2040 | ~20 PLN | ✅ | ❌ | **Best balance** |
| RP2040 Zero | ~25 PLN | ✅ | ❌ | Slightly bigger, same chip |
| Arduino Pro Mini | ~7 PLN | ❌ | ❌ | Cheapest but no USB, old AVR |
| STM32F103 (Blue Pill) | ~30 PLN | ❌ | ❌ | More "pro", steeper learning curve |
| CH32V003 | ~8 PLN | ❌ | ❌ | Ultra cheap RISC-V, poor ecosystem |
| Raspberry Pi Pico | ~17 PLN | ❌ (micro) | ✅ (M2) | Only option with mounting holes, but micro-USB |

**Key finding:** No ultra-small board has BOTH USB-C AND mounting holes.
→ Solution: custom carrier PCB (see below).

---

## 3. Carrier PCB Design (Replaces Plexi Mount)

### Concept

Instead of laser-cut plexi, use a custom PCB as the mounting carrier:
- PCB with footprints for MCU + sensor
- Gold pins / headers → plug modules in (no soldering for assembly)
- 2-4 mounting holes (M2) for mechanical attachment
- Simple traces: power (5V, 3.3V, GND) + I2C (SDA, SCL)
- Optional: USB-C connector routed from MCU

### Dimensions

- Target: ~50×30 mm (enough for XIAO + VL53L1X breakout side by side)
- 4× M2 mounting holes at corners
- Could be as simple as 40×25 mm if layout is tight

### Cost (from JLCPCB/PCBWay, 2026 prices)

| Quantity | PCB cost | Shipping | Per-unit |
|----------|----------|----------|----------|
| 5 pcs | ~8-20 PLN | ~12-20 PLN | ~5-8 PLN |
| 10 pcs | ~15-25 PLN | ~15-25 PLN | ~3-5 PLN |
| 50 pcs | ~30-50 PLN | ~25-40 PLN | ~1-2 PLN |
| 100 pcs | ~40-70 PLN | ~30-50 PLN | ~1 PLN |

**At production quantities (100+), carrier PCB costs ~1 PLN per unit.**
This is CHEAPER than plexi + laser cutting.

### Advantages over plexi

| | Plexi | Carrier PCB |
|---|---|---|
| Cost at 100 units | ~10-30 PLN/unit | ~1-2 PLN/unit |
| Assembly | manual wiring | plug in modules |
| Appearance | "prototype" | "semi-pro" |
| Electrical routing | external wires | PCB traces |
| Mounting | needs separate bracket | double-sided tape or M2 screws |
| EMC | no shielding | ground plane helps slightly |
| Iteration | new laser cut file | new Gerber, same factory |

### Mounting to desk

**Primary: strong double-sided tape (3M VHB or similar)**
- Holds several kg, suitable for ~20g device
- User sticks it under desk in 10 seconds
- Removable without damage (heat gun or dental floss to release)
- Included in dev kit

**Alternative: M2 screws into desk**
- For users who want permanent mounting
- Requires drilling — most users won't bother
- Documented as option, not default

### Module connection: Gold pin headers

- Female headers on carrier PCB
- Male headers soldered on MCU + sensor modules (usually pre-soldered)
- Plug in → done
- Removable if needed (swap MCU, replace sensor)
- No soldering required for end user

---

## 4. Optional Components (Future Expansion)

### 4a. Vibration Motor — Haptic Feedback

Physical buzz under the desk when sitting too long. Works even when user isn't
looking at screen. Unique selling point vs software-only solutions.

**Recommended type: Pancake motor (coin/flat)**
- Flat disc shape, ~10mm diameter, ~3mm thick
- 3-5V operation, ~80-100mA
- Strong vibration in a small form factor
- Mounts flat on PCB or stuck with adhesive
- Price: ~3-8 PLN

Alternative: Cylinder motor (like in phones)
- Slightly bigger, similar vibration
- Easier to find on AliExpress
- Price: ~3-5 PLN

**Driver circuit:**
- NPN transistor (2N2222 or S8050) — MCU GPIO can't drive motor directly
- Flyback diode (1N4148) across motor terminals — protects MCU from back-EMF
- PWM control for variable intensity (gentle reminder → urgent alert)
- Total driver BOM: ~1 PLN

**Firmware integration:**
- GPIO pin → PWM → transistor → motor
- Patterns: short pulse (tap), double pulse (nudge), continuous (urgent)
- Controlled via same alert escalation system as overlay bar

### 4b. Presence/Vibration Sensors — Desk Activity Detection

Detects if user is at the desk by sensing vibrations from typing, mouse movement,
or desk contact. Replaces OS-level keyboard/mouse monitoring (`activity.rs`).

**Three options compared:**

#### Option A: Piezo disc (~1-3 PLN) — CHEAPEST

A thin ceramic disc that generates voltage when flexed/vibrated.
NOT a buzzer — same physical component, but used as a SENSOR (reads voltage
via ADC instead of driving it with signal).

- How it works: typing/mouse movement → desk vibrates → piezo generates
  small voltage → MCU reads via ADC → threshold → "user present"
- Pros: ultra cheap, very sensitive to surface vibrations, analog output
  (can measure vibration intensity)
- Cons: needs ADC + software filtering (high-pass filter to remove DC offset,
  threshold tuning), sensitive to environmental vibration (HVAC, footsteps)
- Best for: prototype/testing phase — cheapest way to validate the concept

#### Option B: SW-420 vibration sensor (~2-6 PLN) — SIMPLEST

Spring-based sensor with digital (HIGH/LOW) output. Built-in comparator.

- How it works: vibration → spring moves → closes circuit → digital HIGH
- Pros: digital output (no ADC needed), dead simple code (`digitalRead`)
- Cons: binary only (no intensity), less sensitive to subtle vibrations,
  "bouncy" output (needs debounce), adjustable threshold via potentiometer
  but not very precise
- Best for: quick "is there any movement?" detection

#### Option C: ADXL345 accelerometer (~10-20 PLN) — MOST PRECISE

3-axis digital accelerometer via I2C/SPI. Measures acceleration on X/Y/Z.

- How it works: detects micro-accelerations from typing, desk bumps,
  even subtle weight shifts
- Pros: very precise, 3-axis (can distinguish typing from desk bump),
  built-in tap/double-tap detection, configurable sensitivity, I2C
  (same bus as VL53L1X — no extra pins needed)
- Cons: more expensive, more complex firmware, overkill for binary
  "present/absent" detection
- Best for: production version — if presence detection proves valuable

**Comparison table:**

| Sensor | Price | Sensitivity | Interface | Complexity | Recommendation |
|--------|-------|-------------|-----------|------------|---------------|
| Piezo disc | 1-3 PLN | HIGH | ADC (analog) | Medium (filtering) | Prototype testing |
| SW-420 | 2-6 PLN | MEDIUM | Digital GPIO | LOW | Quick validation |
| ADXL345 | 10-20 PLN | VERY HIGH | I2C | Medium-High | Production v2 |

**Testing plan:** Order all three (~15-30 PLN total). Test each on breadboard
with current ESP32-C3 prototype. Measure: can it reliably distinguish
"typing at desk" from "empty desk" from "walking past desk"?

### AliExpress Search Guide (piezo ≠ buzzer!)

Searching "piezo" on AliExpress returns BUZZERS (speakers), not sensors.
This is a common naming confusion. Use these exact search terms:

**For vibration SENSORS (what we want):**
- `piezo vibration sensor` or `piezoelectric vibration sensor`
- `piezo disc sensor` or `piezo sensor module`
- `SW-420 vibration sensor module` (digital, with comparator)
- `mini vibration sensor`

**NOT these (these are speakers/buzzers):**
- ~~piezo buzzer~~ → speaker, makes sound
- ~~passive buzzer~~ → speaker
- ~~active buzzer~~ → speaker with built-in oscillator

**How to tell sensor from buzzer:**
| | Sensor | Buzzer |
|---|---|---|
| Look | flat disc or bare plate | round with hole on top |
| Pins | 2-3 pins | 2 pins |
| Description | "vibration sensor", "piezoelectric sensor" | "buzzer", "alarm" |
| Function | generates voltage when vibrated | makes sound when powered |

**For vibration MOTORS (haptic feedback):**
- `vibration motor pancake` or `coin vibration motor`
- `flat vibration motor 3V`
- `cell phone vibration motor`

**For accelerometers:**
- `ADXL345 module` (3-axis, I2C, most common)
- `GY-291 ADXL345` (common breakout board name)

### 4c. Buzzer (~2 PLN) — SKIP

Audio alert (beep). May be annoying in office environment.
Vibration motor is better for non-intrusive feedback.

### 4d. LED (~1 PLN) — SKIP for v1

Visual status indicator on the device itself. Redundant with app UI.
Could be useful in v2 if device is visible (e.g., mounted on desk edge).

### Assessment Summary

| Component | Cost | Value | v1 Dev Kit | v2 | PCB footprint now? |
|-----------|------|-------|:---:|:---:|:---:|
| Vibration motor (pancake) | 3-8 PLN | HIGH | ❌ | ✅ | ✅ YES |
| Piezo sensor | 1-3 PLN | MEDIUM | ❌ | Maybe | ✅ YES |
| ADXL345 | 10-20 PLN | HIGH | ❌ | ✅ | ✅ YES (I2C shared) |
| Transistor driver (for motor) | 1 PLN | required for motor | ❌ | ✅ | ✅ YES |
| Buzzer | 2 PLN | LOW | ❌ | ❌ | ❌ |
| LED | 1 PLN | LOW | ❌ | Maybe | Optional |

**Decision: v1 dev kit = MCU + ToF sensor only. V2 adds vibration motor + presence sensor.**

**PCB design rule:** Include footprints for vibration motor + transistor driver +
ADXL345 (or generic I2C header) on carrier PCB v1. Components not populated
in v1 — but PCB is ready for v2 without redesign. Cost of extra footprints: zero.

---

## 5. Updated BOM (Dev Kit v1)

| Component | Source | Unit cost (100 pcs) |
|-----------|--------|-------------------|
| XIAO RP2040 (or ESP32-C3) | AliExpress | ~20-30 PLN |
| VL53L1X breakout (Grove) | AliExpress | ~15-25 PLN |
| Carrier PCB | JLCPCB | ~1-2 PLN |
| Pin headers (male+female) | AliExpress | ~1-2 PLN |
| 3M VHB tape (cut to size) | Local | ~1-2 PLN |
| USB-C cable (1m) | AliExpress | ~3-5 PLN |
| Anti-static bag + label | AliExpress | ~1-2 PLN |
| **Total BOM** | | **~42-68 PLN (~10-17 EUR)** |

At 49 EUR sale price (~210 PLN):
- BOM: ~55 PLN
- Shipping (EU avg): ~35 PLN
- **Margin: ~120 PLN (~28 EUR) per unit**

At 99 EUR Founder's price (~425 PLN):
- Same BOM + shipping
- **Margin: ~335 PLN (~78 EUR) per unit**

---

## 6. Hardware Decisions Still Open

| Decision | Options | When to decide |
|----------|---------|---------------|
| MCU: keep ESP32-C3 or switch to RP2040? | Both work, RP2040 is cleaner | Before first production batch |
| Carrier PCB design tool | KiCad (free, recommended) | Before ordering PCBs |
| USB cable: include or not? | Include (controls EMC, better UX) | Before production |
| Tape brand/type | 3M VHB recommended | Before production |
| Future: vibration motor? | Yes for v2, footprint on PCB now | PCB design phase |
| Future: piezo sensor? | Maybe for v2, footprint on PCB now | After v1 user feedback |

---

## 7. Production Timeline (for 100 units)

| Step | Duration | Notes |
|------|----------|-------|
| PCB design (KiCad) | 1-2 days | Simple 2-layer board |
| PCB order (JLCPCB) | 5-7 days production + 7-14 days shipping | Order 110 (10 spare) |
| Component order (AliExpress) | 2-4 weeks shipping | Order early, bulk discount |
| Assembly | 2-3 weeks (evenings) | ~15-20 min per unit |
| Testing | 1 week | Plug each into PC, verify reading |
| Packaging + shipping | 1 week | Batch ship via InPost/DPD |
| **Total** | **~6-8 weeks** after components arrive |

Start component order as soon as pre-order threshold (100) is reached.
PCB design can happen in parallel with pre-order collection.
