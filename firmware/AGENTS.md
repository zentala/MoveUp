# USB sensor firmware

Read [README.md](README.md) for supported boards, sensor identity, wiring and
the serial protocol. For guided setup or flashing, read the repository-owned
[moveup-device-setup skill](../.agents/skills/moveup-device-setup/SKILL.md).

The minimum module build combines an RP2040-Zero and a VL53L0X breakout with
four connections. A 3 x 7 cm perfboard is optional mechanical support;
its pad connectivity must be checked rather than assumed. Detailed assembly
photos and the article belong to the companion `desk.zentala.io` repository,
under `hardware/module-sensor/`, linked from its hardware instructions.

Do not confuse this module build with a custom RP2040 PCB or the historical
XIAO ESP32-C3 prototype. Confirm exact board and sensor variants before flashing
or changing pin assignments. A missing sensor is an expected board-only test
condition, not permission to fabricate a distance or reflash repeatedly.

Firmware stays in this repository for now. A separate firmware repository and
flashing through the desktop app are future work, not existing capabilities.
