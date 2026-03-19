#!/bin/bash
# Auto Testing Loop for Overlay Progress Bar
# Tests OPAQUE mode by default (matches what user sees)
# Usage: bash auto-test.sh [opaque|layered]

set -e

# Parse mode argument (default: opaque)
MODE="${1:-opaque}"
if [ "$MODE" != "opaque" ] && [ "$MODE" != "layered" ]; then
    echo "Usage: bash auto-test.sh [opaque|layered]"
    echo "  opaque  - test default OPAQUE mode (GDI, black bg)"
    echo "  layered - test LAYERED mode (UpdateLayeredWindow, transparent)"
    exit 1
fi

PROJECT_DIR="$(pwd)"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
TEST_DIR=".claude/test-runs/${TIMESTAMP}"
LOG_FILE="${TEST_DIR}/overlay.log"
RESULTS_FILE=".claude/test-runs/LATEST-RESULTS.md"
TEST_TIMEOUT=30

echo "=================================================="
echo "Auto Test Loop Started"
echo "=================================================="
echo "Timestamp: $TIMESTAMP"
echo "Mode: $MODE"
echo ""

# Step 1: Create test directory
mkdir -p "$TEST_DIR"

# Step 2: Kill old processes (Windows-compatible)
echo "Cleaning processes..."
taskkill //F //IM "desk.exe" 2>/dev/null || true
taskkill //F //IM "zntl-desk.exe" 2>/dev/null || true
sleep 2

# Step 3: Build code (check if compiles)
echo "Building Rust code..."
cd src-tauri
if ! cargo check --quiet 2>/dev/null; then
    echo "COMPILATION FAILED"
    echo "Cannot proceed with broken code"
    cd ..
    exit 1
fi
cd ..
echo "Code compiles OK"

# Step 4: Set environment and run test
echo "Running test ($TEST_TIMEOUT seconds, mode=$MODE)..."

ENV_VARS="OVERLAY_DEV_MODE=true RUST_LOG=warn"
if [ "$MODE" = "layered" ]; then
    ENV_VARS="OVERLAY_MODE=layered $ENV_VARS"
fi

eval "$ENV_VARS pnpm tauri:dev" > "$LOG_FILE" 2>&1 &
APP_PID=$!

# Progress indicator with timeout
for i in $(seq 1 $TEST_TIMEOUT); do
    echo -n "."
    sleep 1
done
echo ""

# Kill app (Windows-compatible)
taskkill //F //IM "desk.exe" 2>/dev/null || true
taskkill //F //IM "zntl-desk.exe" 2>/dev/null || true
taskkill //F //PID $APP_PID 2>/dev/null || true
sleep 2

echo "Test complete"
echo ""

# Step 5: Parse logs
echo "Analyzing logs..."
echo ""

# Check if log file has content
if [ ! -f "$LOG_FILE" ] || [ ! -s "$LOG_FILE" ]; then
    echo "NO LOGS CAPTURED"
    exit 1
fi

# Extract metrics based on mode
if [ "$MODE" = "opaque" ]; then
    # OPAQUE mode logs: [DEV] Stage N: progress=X%
    DEV_STAGE_COUNT=$(grep -c "\[DEV\] Stage" "$LOG_FILE" 2>/dev/null || echo "0")
    UNIQUE_STAGES=$(grep -o "\[DEV\] Stage [0-9]*" "$LOG_FILE" 2>/dev/null | sort -u | wc -l || echo "0")
    FIRST_STAGE=$(grep "\[DEV\] Stage" "$LOG_FILE" | head -1 || echo "NONE")
    LAST_STAGE=$(grep "\[DEV\] Stage" "$LOG_FILE" | tail -1 || echo "NONE")

    # Also check for WM_PAINT bar_width logs
    PAINT_COUNT=$(grep -c "WM_PAINT" "$LOG_FILE" 2>/dev/null || echo "0")
    FIRST_PAINT=$(grep "WM_PAINT" "$LOG_FILE" | head -1 | grep -o "bar_width=[0-9]*" || echo "NONE")
    LAST_PAINT=$(grep "WM_PAINT" "$LOG_FILE" | tail -1 | grep -o "bar_width=[0-9]*" || echo "NONE")
else
    # LAYERED mode logs: [DRAW] and [DEMO-OVERRIDE]
    DRAW_COUNT=$(grep -c "\[DRAW\]" "$LOG_FILE" 2>/dev/null || echo "0")
    DEMO_OVERRIDE_COUNT=$(grep -c "\[DEMO-OVERRIDE\]" "$LOG_FILE" 2>/dev/null || echo "0")
    FIRST_DRAW=$(grep "\[DRAW\]" "$LOG_FILE" | head -1 | grep -o "bar_width=[0-9]*/[0-9]*" || echo "NONE")
    LAST_DRAW=$(grep "\[DRAW\]" "$LOG_FILE" | tail -1 | grep -o "bar_width=[0-9]*/[0-9]*" || echo "NONE")
    FIRST_DEMO=$(grep "\[DEMO-OVERRIDE\]" "$LOG_FILE" | head -1 | grep -o "progress=[0-9.]*" || echo "NONE")
    LAST_DEMO=$(grep "\[DEMO-OVERRIDE\]" "$LOG_FILE" | tail -1 | grep -o "progress=[0-9.]*" || echo "NONE")
fi

# Common metrics
TIMER_COUNT=$(grep -c "\[TIMER\]" "$LOG_FILE" 2>/dev/null || echo "0")
DEMO_MODE=$(grep -o "demo_mode=true" "$LOG_FILE" | head -1 || echo "NOT FOUND")
VISIBLE=$(grep -o "visible=true" "$LOG_FILE" | head -1 || echo "NOT FOUND")

# Generate results
cat > "$RESULTS_FILE" << EOF
# Test Results - $TIMESTAMP

## Config
- Mode: **$MODE**
- Timeout: **${TEST_TIMEOUT}s**
- Dev mode: **OVERLAY_DEV_MODE=true**

## Common Metrics
- TIMER logs: **$TIMER_COUNT**
- demo_mode: **$DEMO_MODE**
- visible: **$VISIBLE**

EOF

if [ "$MODE" = "opaque" ]; then
    cat >> "$RESULTS_FILE" << EOF
## OPAQUE Mode Metrics
- DEV Stage logs: **$DEV_STAGE_COUNT**
- Unique stages: **$UNIQUE_STAGES**
- First stage: **$FIRST_STAGE**
- Last stage: **$LAST_STAGE**
- WM_PAINT logs: **$PAINT_COUNT**
- First bar_width: **$FIRST_PAINT**
- Last bar_width: **$LAST_PAINT**

## Analysis

### Are dev stages cycling?
EOF
    if [ "$DEV_STAGE_COUNT" -gt 0 ]; then
        echo "YES - Found $DEV_STAGE_COUNT stage logs" >> "$RESULTS_FILE"
        if [ "$UNIQUE_STAGES" -gt 1 ]; then
            echo "YES - $UNIQUE_STAGES unique stages detected (bar width is changing)" >> "$RESULTS_FILE"
        else
            echo "NO - Only 1 unique stage (bar width NOT changing)" >> "$RESULTS_FILE"
            echo "   -> Problem: stage calculation not cycling" >> "$RESULTS_FILE"
        fi
    else
        echo "NO - No [DEV] Stage logs found" >> "$RESULTS_FILE"
        echo "   -> Problem: Dev mode not active or log format changed" >> "$RESULTS_FILE"
    fi

    echo "" >> "$RESULTS_FILE"
    echo "### Is bar_width changing?" >> "$RESULTS_FILE"
    if [ "$PAINT_COUNT" -gt 0 ]; then
        echo "YES - Found $PAINT_COUNT WM_PAINT logs" >> "$RESULTS_FILE"
        echo "   First: $FIRST_PAINT" >> "$RESULTS_FILE"
        echo "   Last: $LAST_PAINT" >> "$RESULTS_FILE"
    else
        echo "NO - No WM_PAINT logs found (may be normal if logging is minimal)" >> "$RESULTS_FILE"
    fi
else
    cat >> "$RESULTS_FILE" << EOF
## LAYERED Mode Metrics
- DRAW logs: **$DRAW_COUNT**
- DEMO-OVERRIDE logs: **$DEMO_OVERRIDE_COUNT**
- First DRAW bar_width: **$FIRST_DRAW**
- Last DRAW bar_width: **$LAST_DRAW**
- First DEMO progress: **$FIRST_DEMO**
- Last DEMO progress: **$LAST_DEMO**

## Analysis

### Is demo progress cycling?
EOF
    if [ "$DEMO_OVERRIDE_COUNT" -gt 0 ]; then
        echo "YES - Found $DEMO_OVERRIDE_COUNT DEMO-OVERRIDE logs" >> "$RESULTS_FILE"
        echo "   First: $FIRST_DEMO" >> "$RESULTS_FILE"
        echo "   Last: $LAST_DEMO" >> "$RESULTS_FILE"
    else
        echo "NO - No DEMO-OVERRIDE logs found" >> "$RESULTS_FILE"
    fi

    echo "" >> "$RESULTS_FILE"
    echo "### Is bar_width reflecting progress?" >> "$RESULTS_FILE"
    if [ "$DRAW_COUNT" -gt 0 ]; then
        echo "YES - Found $DRAW_COUNT DRAW logs" >> "$RESULTS_FILE"
        echo "   First: $FIRST_DRAW" >> "$RESULTS_FILE"
        echo "   Last: $LAST_DRAW" >> "$RESULTS_FILE"
    else
        echo "NO - No DRAW logs found" >> "$RESULTS_FILE"
    fi
fi

echo "" >> "$RESULTS_FILE"
echo "## Next Steps" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

if ! echo "$DEMO_MODE" | grep -q "true"; then
    echo "1. Fix dev mode activation (demo_mode not true)" >> "$RESULTS_FILE"
elif [ "$MODE" = "opaque" ] && [ "$DEV_STAGE_COUNT" -eq 0 ]; then
    echo "1. Fix OPAQUE dev stage logging (no [DEV] Stage logs)" >> "$RESULTS_FILE"
elif [ "$MODE" = "opaque" ] && [ "$UNIQUE_STAGES" -le 1 ]; then
    echo "1. Fix stage cycling (only 1 unique stage)" >> "$RESULTS_FILE"
elif [ "$MODE" = "layered" ] && [ "$DRAW_COUNT" -eq 0 ]; then
    echo "1. Fix draw_layered_frame not being called" >> "$RESULTS_FILE"
else
    echo "1. All systems working. Ask user for visual feedback." >> "$RESULTS_FILE"
fi

# Print results
echo "=================================================="
echo "Results saved to: $RESULTS_FILE"
echo "=================================================="
echo ""
cat "$RESULTS_FILE"
echo ""
echo "Log file: $LOG_FILE"
