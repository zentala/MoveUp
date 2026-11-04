---
name: moveup-device-setup
description: Guide a beginner through identifying, checking, wiring and flashing a MoveUp USB desk sensor. Use when someone has bought parts, asks what to connect next, or wants to prepare or test an RP2040-Zero. Supports a board-only check without flashing or a connected sensor.
---

# Prepare a MoveUp USB sensor

Read `firmware/README.md` from the repository root before connecting or flashing.
The supported module build is **Waveshare RP2040-Zero + blue VL53L0X breakout**.
A 3 x 7 cm perfboard is an optional mounting carrier, not another microcontroller.
The minimum assembly is two modules and four electrical connections.
Other boards or VL53L1X sensors require their own board target or driver.

## Start with what the person has

Identify the board from its markings or a photo and ask only about missing parts
that affect the next step. Discover the current USB device and serial port;
never assume a port name, drive letter or saved USB identifier from another machine.
Explain the next physical step in plain language, one stage at a time.

Choose the requested outcome:

- **"Does this board run a program?"** Open its identified USB serial port at
  115200 baud with DTR enabled, send `PING\n` and `STATUS\n`, and collect replies
  for up to five seconds. Close the port afterward. Do not reflash a working
  program merely to demonstrate execution. `DEVICE: zntl-desk-sensor v1` plus
  `BOARD: RP2040-Zero` proves this firmware runs and communicates. A sensor error
  is expected if nothing is wired. A bootloader disk alone proves only USB boot.
- **"Prepare/flash my device"** Continue with the build and flash steps below.
- **"Assemble the sensor"** Explain wiring and mounting, then test real readings.
  If parts have not arrived, provide the instructions and record the pending
  assembly; do not ask them to wire parts they do not have.

## Build and flash an RP2040-Zero

1. Use an existing Arduino CLI if available. Otherwise obtain the official CLI
   for the host OS from Arduino, unpack it into a task-local tools directory,
   and keep its data, downloads and libraries there. Do not change global IDE
   settings or install USB drivers just because the bootloader's extra interface
   lacks a driver; normal UF2 copying uses its working mass-storage interface.
2. Configure the additional board index:
   `https://github.com/earlephilhower/arduino-pico/releases/download/global/package_rp2040_index.json`.
   With a local YAML config, run the CLI with `--config-file <config>` throughout.
   Set `directories.data`, `directories.downloads` and `directories.user` to
   subdirectories of the chosen tools directory.
3. Install the board core and sensor library. The tested combination is
   Arduino-Pico **6.3.1** and Adafruit VL53L0X **1.2.5**:

   ```text
   arduino-cli core update-index
   arduino-cli core install rp2040:rp2040@6.3.1
   arduino-cli lib install Adafruit_VL53L0X@1.2.5
   arduino-cli compile --fqbn rp2040:rp2040:waveshare_rp2040_zero --output-dir <output-directory> firmware/tof_reader
   ```

4. Observe a successful compile and the generated `tof_reader.ino.uf2`. Do not
   substitute an arbitrary downloaded binary or an ESP32 build. If compilation
   fails, diagnose that failure before any flash attempt.
5. Ask the person to hold **BOOT/BOOTSEL while connecting USB**, then release
   it. Discover the new `RPI-RP2` volume and read `INFO_UF2.TXT`; it should identify
   Raspberry Pi RP2. This file does not identify the exact board variant, which
   was established from markings. If several boards match, unplug the others
   or identify the selected one before writing.
6. For an authorized setup request, copy the built UF2 to that verified volume.
   Expect it to disappear and a USB serial port to appear. Discover the new port
   and perform the PING/STATUS check above. Do not use a hard-coded COM port or
   disk letter, format the volume, erase the chip, or flash other connected boards.

If no bootloader appears, try a known USB data cable and a direct USB port,
then repeat the physical BOOT procedure once. If still absent, report what the
host actually sees; do not declare the board dead from that observation alone.
If a serial port is busy, identify its owner; never kill MoveUp or another
program to take the port. Coordinate release of that specific port instead.

## Wire and mount the modules

Disconnect USB before handling wires or soldering. Connect **VIN -> 3V3,
GND -> GND, SDA -> GP26, SCL -> GP27**. Read silkscreen labels rather than header
position numbers. Do not connect sensor VIN to 5V for this build. Leave other
sensor pads unused. Never substitute the ESP32-C3 GPIO6/GPIO7 mapping.

Use red for 3V3/VIN, black for GND, white for SDA and yellow for SCL, following
the Grove colour convention rather than a universal I2C colour standard.
The current RP2040 sketch uses `Wire1` (I2C1) for GP26/GP27. Older GP0/GP1
builds need rebuilding and flashing before moving signal wires; never change
only the diagram. `STATUS` must report `I2C: SDA=26 SCL=27`. A request to revise
instructions or a model alone does not authorize flashing attached hardware.

Choose the mounting arrangement before soldering: a carrier with USB at its end
or side, or two independently mounted modules without a carrier. The companion
`desk.zentala.io` repository maintains the assembly article and models at
`astro/src/content/blog/module-sensor-assembly.mdx` and `hardware/module-sensor/`.
Use the chosen layout's own underside map; hole IDs do not transfer between
arrangements. The named electrical connections above remain the same.

For a carrier, dry-fit modules, headers and the actual USB plug before cutting.
Use the current normal downward-header arrangement: plastic spacers below the
modules, short solder ends above them, and the long ends through the carrier.
Trim excess ends on the carrier's back before routing. Verify both leg lengths,
spacer height and clearance for the components underneath the controller;
the 2.54 mm pitch alone does not establish fit. The earlier long-up arrangement
was superseded by the owner's assembly decision. Check pad connectivity with
a continuity meter: isolated pads
are not automatically joined. Short tinned bridges may follow clear corridors;
use insulation wherever a route could touch another net, pad or pin. Four short
insulated wires are also a valid choice.

Without a carrier, solder four flexible insulated wires directly to the named
pads. Headers are not required. Support and insulate each module separately
before using double-sided tape; keep tape away from pads and optical windows.
In either arrangement, keep USB and BOOT/RESET accessible, the sensor aimed at
the floor, and wires outside its optical clearance. Do not press exposed joints
against the desk or a metal frame. A separate power LED is optional and needs
a series resistor; do not add a buzzer from a size illustration alone.

After soldering, check for solder bridges and a power-to-ground short before
reconnecting USB. A chosen layout on perfboard is a suggested arrangement until
the physical assembly has been inspected and tested.

## Test the assembled device

At 115200 baud, expect `STATUS: OK` and repeated `distance: <n> mm` lines. Check
that distance changes when a target moves, then test the actual desk-to-floor
range and floor surface. Close the monitor before letting MoveUp use the port.
Verify recognition and real readings in MoveUp separately; do not change app
calibration, interrupt another sensor, or replace the running desktop app unless
that is part of the user's request.

For a missing sensor, recheck its label, four wires and power with USB removed.
Do not repeatedly flash the same working firmware. A VL53L1X needs a matching
driver; it does not become a VL53L0X by sharing an I2C address. Invalid readings
are errors, not proof of desk height.

Report separately: USB boot detected, firmware compiled, flash completed,
program responding, sensor measurements observed, MoveUp integration checked.
Mark unobserved stages as not tested and give the next concrete physical step.
Do not promise flashing through the app; that feature is planned.

## Maintain this skill

Its editable source is this repository, not a global installed copy. Authors
read the adjacent `AGENTS.md` before editing. When a fresh-session trial exposes
a missing step or repeated confusion, record the observation in the existing
project backlog, correct the owning instruction, and retest that step. One
successful board does not establish support for other sensor or MCU variants.
