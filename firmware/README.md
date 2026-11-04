# MoveUp firmware

The sensor sketch that MoveUp talks to over USB serial. One sketch:
[`tof_reader/tof_reader.ino`](tof_reader/tof_reader.ino).

## Guided setup with an agent

From a checkout containing this guide, start your coding agent and ask:

> I bought an RP2040-Zero, a VL53L0X and a perfboard. Use
> moveup-device-setup to check what I have and guide me through assembly,
> firmware flashing and tests. If the sensor is not connected, check only
> whether the board runs its current program.

The repository entrypoint links the
[device setup skill](../.agents/skills/moveup-device-setup/SKILL.md), so an agent
can read it even if its client does not automatically discover repository skills.
For a manual test of the existing program, send `PING` and `STATUS` over its
identified USB serial port, using 115200 baud and DTR enabled. Discover the port
on your own machine; port numbers can change. Close the monitor afterward.

`PING` identification and the `BOARD: RP2040-Zero` response prove that the
controller executes this program and communicates over USB. They do not test
all GPIO, sensor measurements or a soldered assembly. A sensor error is expected
without a sensor. Do not reflash a responsive board just to prove that it runs.

The two-module assembly reference and source photos are maintained in the
companion `desk.zentala.io` repository at `hardware/module-sensor/`. It covers
an optional perfboard carrier and direct wiring without one. All arrangements
use the same four named connections below; their physical hole positions differ.
Firmware remains here for now; a separate repository and flashing from MoveUp
are future work.

## RP2040-Zero with the blue VL53L0X module

Select **Waveshare RP2040 Zero** in the Earle Philhower Arduino-Pico board
package, and install **Adafruit_VL53L0X**. The same sketch selects the RP2040
I2C setup at compile time. It uses USB CDC serial, not UART.

Disconnect USB before wiring. Use board labels, not header pin numbers.
With the RP2040-Zero component side facing you and USB at the top, the left
edge reads 5V, GND, 3V3, GP29, GP28, GP27, GP26, GP15, GP14.
GP26 and GP27 are on the same edge as power; use **3V3**, not 5V.

| Sensor label | RP2040-Zero label |
|---|---|
| VIN | 3V3 |
| GND | GND |
| SDA | GP26 (marked 26), white wire |
| SCL | GP27 (marked 27), yellow wire |

Use red for 3V3/VIN and black for GND. These colours follow the Grove
convention; I2C itself does not prescribe wire colours. Always check labels.
The RP2040 build uses **Wire1 (I2C1)**, including the sensor driver, because
GP26/GP27 are I2C1 pins. The ESP32-C3 mapping is unchanged.

**Earlier RP2040 builds used GP0/GP1.** Rebuild and flash this revision before
moving those signal wires to GP26/GP27. A firmware update does not move physical
wires, and rewiring alone does not update firmware. With USB disconnected,
check the intended wiring; after flashing, `STATUS` must report
`I2C: SDA=26 SCL=27` before expecting sensor readings.

Leave the remaining sensor pins disconnected. This wiring is for the pictured
VL53L0X breakout; a different module needs its own pinout.

For the first upload, hold BOOT while connecting USB and copy the compiled
UF2 to the RPI-RP2 drive. The board reboots as a USB serial device.
At 115200 baud, `PING` followed by a newline returns
`DEVICE: zntl-desk-sensor v1`; `STATUS` reports the board, I2C pins and sensor
initialization state. The firmware responds without the sensor and retries
initialization every three seconds. Failed measurements emit errors, not
distance readings. Disconnect USB before changing wires, then reconnect.

Arduino CLI build, after configuring the Arduino-Pico package index:

```powershell
arduino-cli core install rp2040:rp2040
arduino-cli lib install Adafruit_VL53L0X
arduino-cli compile --fqbn rp2040:rp2040:waveshare_rp2040_zero --output-dir build/rp2040 firmware/tof_reader
```

References: [Arduino-Pico board mapping](https://github.com/earlephilhower/arduino-pico/blob/master/variants/waveshare_rp2040_zero/pins_arduino.h)
and [Wire API](https://arduino-pico.readthedocs.io/en/latest/wire.html).

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
output: `STATUS: OK` means initialization succeeded. `ERROR:
VL53L0X not found or failed to start` on a VL53L1X module means you need the VL53L1X library and a
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
| `ERROR: VL53L0X not found or failed to start` | sensor not initialized — check wiring or sensor library (see above) |
| `ERROR: out of range` | invalid measurement: nonzero sensor range status or distance ≥ 8190 mm; check target distance, reflectivity and alignment |
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
| host → board | `STATUS` | ask for board, I2C pins and sensor initialization state |
