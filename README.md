# MoveUp

MoveUp helps you use a sit/stand desk with more intention. It reads the height of
your desk, recognises sitting and standing sessions, and gives gentle visual
reminders when it is time to change position or take a break.

## Why use it

Long, uninterrupted desk time is easy to miss. MoveUp makes the pattern visible:

- shows the current desk state and height in the Windows tray;
- tracks sitting, standing, walking and away time across the day;
- gives configurable reminders and a subtle screen overlay;
- keeps a local history and an Analyst view of your day;
- can show a read-only live dashboard in a browser on your local network.

## What you need

MoveUp is a Windows desktop application for a motorised sit/stand desk. Its full
automatic mode needs a compatible VL53L1X distance sensor and controller connected
to the computer by USB. The app can also be explored with simulated readings while
the hardware is not connected.

The controller is hardware integration, not a requirement to understand the app.
It is not a cloud service and MoveUp keeps its session data locally on your computer.

## Current status

MoveUp is in active development and is not yet distributed as a public installer.
When it is released, users will install a signed Windows package. They will not
need Cargo, PM3, a local domain, or a Windows security exception.

## Limits

- The browser dashboard is read-only and the desktop app must be running.
- Automatic desk detection depends on a correctly installed and calibrated sensor.
- The app is designed around one desk and one person, not shared workplace fleet
  management.

## For developers

Development setup, testing, local PM3 use and Windows Application Control guidance
live in [CONTRIBUTING.md](CONTRIBUTING.md).
