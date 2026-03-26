#!/usr/bin/env bash
# tauri-dev.sh — Unified tauri dev launcher with process guard
#
# Usage:
#   bash scripts/tauri-dev.sh                  # live mode (default, real sensor)
#   bash scripts/tauri-dev.sh --demo           # cycling demo animation
#   bash scripts/tauri-dev.sh --mock           # simulated sit/stand
#   bash scripts/tauri-dev.sh --force          # live + auto-kill old instance
#   bash scripts/tauri-dev.sh --demo --force   # demo + auto-kill
#
# Flags:
#   --live    OVERLAY_DATA=live (default)
#   --mock    OVERLAY_DATA=mock
#   --demo    OVERLAY_DATA=demo
#   --force   Auto-kill previous desk.exe without prompting

set -euo pipefail

PROCESS_NAME="desk.exe"
OVERLAY_DATA=""
FORCE_KILL=0

# Parse flags
for arg in "$@"; do
  case "$arg" in
    --)      ;; # pnpm passes "--" separator, ignore it
    --live)  OVERLAY_DATA="live" ;;
    --mock)  OVERLAY_DATA="mock" ;;
    --demo)  OVERLAY_DATA="demo" ;;
    --force) FORCE_KILL=1 ;;
    *)       echo "Unknown flag: $arg"; exit 1 ;;
  esac
done

# --- Process guard ---
if tasklist //FI "IMAGENAME eq $PROCESS_NAME" 2>/dev/null | grep -qi "$PROCESS_NAME"; then
  PID=$(tasklist //FI "IMAGENAME eq $PROCESS_NAME" //FO CSV //NH 2>/dev/null \
    | head -1 | cut -d',' -f2 | tr -d '"' | tr -d ' ')

  echo ""
  echo "========================================"
  echo "  Previous instance detected!"
  echo "  $PROCESS_NAME is running (PID: $PID)"
  echo "========================================"
  echo ""

  if [[ "$FORCE_KILL" == "1" ]]; then
    echo "  --force: killing $PROCESS_NAME..."
    taskkill //F //IM "$PROCESS_NAME" > /dev/null 2>&1 || true
    sleep 1
    echo "  Killed. Starting new build."
    echo ""
  else
    echo "  [k] Kill old process and start new one (default in 10s)"
    echo "  [s] Skip — keep old process, abort build"
    echo ""
    read -t 10 -p "  Choice [k/s]: " CHOICE || CHOICE="k"

    case "${CHOICE,,}" in
      s|skip)
        echo ""
        echo "  Keeping old instance. Build aborted."
        exit 1
        ;;
      *)
        echo ""
        echo "  Killing $PROCESS_NAME (PID: $PID)..."
        taskkill //F //IM "$PROCESS_NAME" > /dev/null 2>&1 || true
        sleep 1
        echo "  Killed. Starting new build."
        echo ""
        ;;
    esac
  fi
fi

# --- Launch tauri dev ---
# tauri CLI is a devDependency — must invoke via pnpm exec
if [[ -n "$OVERLAY_DATA" ]]; then
  export OVERLAY_DATA
fi
exec pnpm exec tauri dev
