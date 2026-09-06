#!/usr/bin/env node
// Verifies E017-T06: firmware/README.md exists and covers board, sensor,
// flashing, success output and the USB-C cable warning.
import { existsSync, readFileSync } from "node:fs";

const README = "firmware/README.md";
const SKETCH = "firmware/tof_reader/tof_reader.ino";
const problems = [];

if (!existsSync(README)) {
  console.error("FAIL: 1 problem(s)");
  console.error(" - " + README + " does not exist");
  process.exit(1);
}

const text = readFileSync(README, "utf8");

// Required content, keyed by what the reader must be able to do.
const required = [
  ["board name", /XIAO ESP32-C3/],
  ["sensor + mounting", /pointing straight down at the floor/i],
  ["I2C pinout", /GPIO6.*D4/s],
  ["Arduino IDE flashing", /Arduino IDE/],
  ["ESP32 board package URL", /package_esp32_index\.json/],
  ["sensor library", /Adafruit_VL53L0X/],
  ["PlatformIO flashing", /platformio|PlatformIO/],
  ["baud rate", /115200/],
  ["success marker", /DEVICE: zntl-desk-sensor v1/],
  ["PING command", /PING/],
  ["cable warning heading", /Cable sensitivity/],
  ["cable cause 1 — charge-only", /Charge-only cables/],
  ["cable cause 2 — CC resistors", /5\.1k CC resistors/],
  ["cable cause 3 — voltage drop", /28 AWG/],
  ["cable diagnosis", /SERIALCOMM/],
];

for (const [label, pattern] of required) {
  if (!pattern.test(text)) {
    problems.push("missing " + label + " (" + pattern + ")");
  }
}

// The link to the sketch must resolve.
if (!existsSync(SKETCH)) {
  problems.push(SKETCH + " does not exist — README links to it");
}
if (!text.includes("tof_reader/tof_reader.ino")) {
  problems.push("README does not link to " + SKETCH);
}

// Stale product identity must not creep back in.
for (const stale of ["zntl-tray", "AppData\\Local\\zntlDesk"]) {
  if (text.includes(stale)) {
    problems.push("stale identity string present: " + stale);
  }
}

if (problems.length > 0) {
  console.error("FAIL: " + problems.length + " problem(s)");
  for (const p of problems) console.error(" - " + p);
  process.exit(1);
}

console.log(
  "PASS: " + README + " covers " + required.length + " required topics, sketch link resolves",
);
process.exit(0);
