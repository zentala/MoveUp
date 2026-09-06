# MoveUp firmware

The sensor sketch that MoveUp talks to over USB serial. One sketch:
[`tof_reader/tof_reader.ino`](tof_reader/tof_reader.ino).

## Hardware

| Part | What |
|---|---|
| Board | Seeed Studio XIAO ESP32-C3 |
| Sensor | Grove Time-of-Flight distance sensor, on I2C |
| Wiring | Grove I2C — `SDA = GPIO6 (D4)`, `SCL = GPIO7 (D5)` |
| Mounting | Under the desk, pointing straight down at the floor |
| USB identity | `VID_303A&PID_1001`, composite: `MI_00` → `usbser` → a COM port, `MI_02` → WinUSB (JTAG) |

Desk height is derived, not measured directly:

```
desk_height = sensor_reading_mm - desk_thickness_mm
```

**Sensor part number — check before you buy.** The product docs call the sensor
a Grove VL53L1X v2, but this sketch drives it with the `Adafruit_VL53L0X`
library and the `VL53L0X_I2C_ADDR` address. Flash the sketch and read the serial
output: `Sensor OK` means the library matches the board you have. `ERROR:
VL53L0X not found` on a VL53L1X module means you need the VL53L1X library and a
matching sketch instead.

## Flashing

### Arduino IDE

1. Install the ESP32 board package. **File → Preferences → Additional Boards
   Manager URLs**, add:

   ```
   https://raw.githubusercontent.com/espressif/arduino-esp32/gh-pages/package_esp32_index.json
   ```

   Then **Tools → Board → Boards Manager**, search `esp32`, install
   *esp32 by Espressif Systems*.
2. **Tools → Board → ESP32 Arduino → XIAO_ESP32C3**.
3. **Tools → Manage Libraries**, install **Adafruit_VL53L0X** by Adafruit
   (accept its dependencies).
4. Plug the board in and pick its port under **Tools → Port**.
5. Open `tof_reader/tof_reader.ino` and click **Upload**.

If upload fails to start, put the board in bootloader mode: hold **BOOT**, tap
**RESET**, release **BOOT**, then upload again.

### PlatformIO

`platformio.ini` for this sketch:

```ini
[env:seeed_xiao_esp32c3]
platform = espressif32
board = seeed_xiao_esp32c3
framework = arduino
monitor_speed = 115200
lib_deps = adafruit/Adafruit_VL53L0X
```

Then `pio run -t upload` and `pio device monitor`.

## Confirming it works

Open a serial monitor at **115200 baud**. A healthy board prints, in order:

```
DEVICE: zntl-desk-sensor v1
VL53L0X Distance Sensor — XIAO ESP32-C3
Sensor OK
STATUS: OK
distance: 1043 mm
distance: 1042 mm
```

The first line is what MoveUp auto-detects the board by — no `DEVICE:` line, no
detection. Send `PING\n` at any time and the board answers with the same
`DEVICE:` line; that is how the app probes a port it is unsure about.

Failure lines:

| Output | Means |
|---|---|
| `ERROR: VL53L0X not found` | sensor not on I2C — check Grove wiring, or wrong sensor library (see above) |
| `ERROR: out of range` | reading ≥ 8190 mm — nothing within range, sensor aimed at empty space |
| nothing at all | see the cable section below before touching the sketch |

## [CRITICAL] Cable sensitivity — most USB-C cables do NOT work with this board

The XIAO ESP32-C3 uses *native* USB (no CH340/CP2102 bridge), so it is far pickier
than a classic Arduino. Three overlapping causes, all observed 2026-09-06:

1. Charge-only cables (VBUS+GND, no D+/D-) — board lights up, host sees nothing,
   and Windows enumerates **zero** COM ports.
2. C-to-C links depend on the board's 5.1k CC resistors; flipping the plug 180°
   or using an A-to-C cable often fixes a link that refuses to come up.
3. Voltage drop on thin (28 AWG) or long cables — the ESP32-C3 plus VL53L1X
   browns out mid-enumeration, producing a `DEVICE connected` / `DEVICE lost`
   loop within the same second, audible as repeated Windows plug/unplug chimes.

Diagnosis order when the app reports no sensor: check for a COM port at all
(`HKLM\HARDWARE\DEVICEMAP\SERIALCOMM`; empty = cable or power, not software),
then check `events.log` for connect/lost churn. Prefer a short (<=1 m) A-to-C
cable straight into the motherboard, bypassing USB hubs.

## Serial protocol

MoveUp reads lines at 115200 baud. Everything is plain text, newline-terminated.

| Direction | Line | Meaning |
|---|---|---|
| board → host | `DEVICE: zntl-desk-sensor v1` | identity, on boot and in reply to `PING` |
| board → host | `STATUS: OK` | sensor initialised |
| board → host | `distance: <n> mm` | one reading, ~10 Hz |
| board → host | `ERROR: <text>` | sensor fault or out of range |
| host → board | `PING` | ask for the identity line |
