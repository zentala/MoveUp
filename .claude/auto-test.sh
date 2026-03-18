#!/bin/bash
# Auto Testing Loop
# Runs test harness, parses logs, analyzes results, reports findings
# Claude runs this internally to self-test and iterate

set -e

PROJECT_DIR="$(pwd)"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
TEST_DIR=".claude/test-runs/${TIMESTAMP}"
LOG_FILE="${TEST_DIR}/overlay.log"
RESULTS_FILE=".claude/test-runs/LATEST-RESULTS.md"

echo "=================================================="
echo "🤖 Auto Test Loop Started"
echo "=================================================="
echo "Timestamp: $TIMESTAMP"
echo ""

# Step 1: Create test directory
mkdir -p "$TEST_DIR"

# Step 2: Kill old processes
echo "📁 Cleaning processes..."
pkill -f "tauri:dev" || true
sleep 2

# Step 3: Build code (check if compiles)
echo "🔨 Building Rust code..."
cd src-tauri
if ! cargo check --quiet 2>/dev/null; then
    echo "❌ COMPILATION FAILED"
    echo "Cannot proceed with broken code"
    cd ..
    exit 1
fi
cd ..
echo "✅ Code compiles"

# Step 4: Run test
echo "🧪 Running test (30 seconds)..."
OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true RUST_LOG=warn pnpm tauri:dev > "$LOG_FILE" 2>&1 &
APP_PID=$!

# Progress indicator
for i in {1..30}; do
    echo -n "."
    sleep 1
done
echo ""

# Kill app
kill $APP_PID 2>/dev/null || true
sleep 2

echo "✅ Test complete"
echo ""

# Step 5: Parse logs
echo "📊 Analyzing logs..."
echo ""

# Check if log file has content
if [ ! -f "$LOG_FILE" ] || [ ! -s "$LOG_FILE" ]; then
    echo "❌ NO LOGS CAPTURED"
    exit 1
fi

# Extract key metrics
TIMER_COUNT=$(grep -c "\[TIMER\]" "$LOG_FILE" 2>/dev/null || echo "0")
DEMO_OVERRIDE_COUNT=$(grep -c "\[DEMO-OVERRIDE\]" "$LOG_FILE" 2>/dev/null || echo "0")
DRAW_COUNT=$(grep -c "\[DRAW\]" "$LOG_FILE" 2>/dev/null || echo "0")
DEMO_MODE=$(grep -o "demo_mode=true" "$LOG_FILE" | head -1 || echo "NOT FOUND")
VISIBLE=$(grep -o "visible=true" "$LOG_FILE" | head -1 || echo "NOT FOUND")

# Sample progress values
FIRST_DEMO=$(grep "\[DEMO-OVERRIDE\]" "$LOG_FILE" | head -1 | grep -o "progress=[0-9.]*" || echo "NONE")
LAST_DEMO=$(grep "\[DEMO-OVERRIDE\]" "$LOG_FILE" | tail -1 | grep -o "progress=[0-9.]*" || echo "NONE")

# Sample bar widths
FIRST_DRAW=$(grep "\[DRAW\]" "$LOG_FILE" | head -1 | grep -o "bar_width=[0-9]*/[0-9]*" || echo "NONE")
LAST_DRAW=$(grep "\[DRAW\]" "$LOG_FILE" | tail -1 | grep -o "bar_width=[0-9]*/[0-9]*" || echo "NONE")

# Generate results
cat > "$RESULTS_FILE" << EOF
# Test Results — $TIMESTAMP

## Metrics
- TIMER logs (every 60 frames): **$TIMER_COUNT**
- DEMO-OVERRIDE logs (every frame): **$DEMO_OVERRIDE_COUNT**
- DRAW logs (every frame): **$DRAW_COUNT**

## Environment
- demo_mode: **$DEMO_MODE**
- visible: **$VISIBLE**

## Progress Values
- First DEMO entry: **$FIRST_DEMO**
- Last DEMO entry: **$LAST_DEMO**

## Bar Width
- First DRAW: **$FIRST_DRAW**
- Last DRAW: **$LAST_DRAW**

## Analysis

### Is demo_mode working?
EOF

if echo "$DEMO_MODE" | grep -q "true"; then
    echo "✅ YES - demo_mode=true detected" >> "$RESULTS_FILE"
else
    echo "❌ NO - demo_mode not set to true" >> "$RESULTS_FILE"
    echo "   → Problem: Env var OVERLAY_DEMO_MODE not read correctly" >> "$RESULTS_FILE"
fi

echo "" >> "$RESULTS_FILE"
echo "### Is frame_count incrementing?" >> "$RESULTS_FILE"
if [ "$TIMER_COUNT" -gt 0 ]; then
    echo "✅ YES - Found $TIMER_COUNT TIMER logs (~1 per second)" >> "$RESULTS_FILE"
else
    echo "❌ NO - No TIMER logs found" >> "$RESULTS_FILE"
    echo "   → Problem: WM_TIMER not firing or logging disabled" >> "$RESULTS_FILE"
fi

echo "" >> "$RESULTS_FILE"
echo "### Is demo progress cycling?" >> "$RESULTS_FILE"
if [ "$DEMO_OVERRIDE_COUNT" -gt 0 ]; then
    echo "✅ YES - Found $DEMO_OVERRIDE_COUNT DEMO-OVERRIDE logs" >> "$RESULTS_FILE"
    echo "   First value: $FIRST_DEMO" >> "$RESULTS_FILE"
    echo "   Last value: $LAST_DEMO" >> "$RESULTS_FILE"

    if echo "$FIRST_DEMO" | grep -q "0.00" && echo "$LAST_DEMO" | grep -q "25.00\|50.00\|75.00\|100.00"; then
        echo "   ✅ Progress changed from 0% to higher → CYCLING WORKS" >> "$RESULTS_FILE"
    elif echo "$FIRST_DEMO" | grep -q "100.00" && echo "$LAST_DEMO" | grep -q "100.00"; then
        echo "   ❌ Progress stuck at 100% → NOT CYCLING" >> "$RESULTS_FILE"
        echo "      → Problem: Progress overwrite not working (tray_controller still overwrites)" >> "$RESULTS_FILE"
    fi
else
    echo "❌ NO - No DEMO-OVERRIDE logs found" >> "$RESULTS_FILE"
    echo "   → Problem: draw_layered_frame not executing or demo_mode false" >> "$RESULTS_FILE"
fi

echo "" >> "$RESULTS_FILE"
echo "### Is bar_width reflecting progress?" >> "$RESULTS_FILE"
if [ "$DRAW_COUNT" -gt 0 ]; then
    echo "✅ YES - Found $DRAW_COUNT DRAW logs" >> "$RESULTS_FILE"
    echo "   First: $FIRST_DRAW" >> "$RESULTS_FILE"
    echo "   Last: $LAST_DRAW" >> "$RESULTS_FILE"

    if echo "$FIRST_DRAW" | grep -q "1/1920" && echo "$LAST_DRAW" | grep -q "1920/1920\|1/1920"; then
        echo "   ❌ Width stuck at 1px or 1920px → NOT CHANGING" >> "$RESULTS_FILE"
    else
        echo "   ✅ Width changing → RENDERING WORKS" >> "$RESULTS_FILE"
    fi
else
    echo "❌ NO - No DRAW logs found" >> "$RESULTS_FILE"
    echo "   → Problem: draw_layered_frame not called" >> "$RESULTS_FILE"
fi

echo "" >> "$RESULTS_FILE"
echo "## Next Steps" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Determine what to fix
if ! echo "$DEMO_MODE" | grep -q "true"; then
    echo "1. Fix env var reading (demo_mode not true)" >> "$RESULTS_FILE"
elif [ "$DRAW_COUNT" -eq 0 ]; then
    echo "1. Fix draw_layered_frame not being called" >> "$RESULTS_FILE"
elif echo "$LAST_DEMO" | grep -q "100.00"; then
    echo "1. Fix progress override (stuck at 100%)" >> "$RESULTS_FILE"
    echo "   → Remove WM_TIMER demo logic, keep draw_layered_frame override" >> "$RESULTS_FILE"
elif echo "$LAST_DRAW" | grep -q "1920/1920\|1/1920"; then
    echo "1. Fix bar_width calculation (not reflecting progress)" >> "$RESULTS_FILE"
else
    echo "1. ✅ All systems working! Ask user for visual feedback." >> "$RESULTS_FILE"
fi

# Print results
echo "=================================================="
echo "Results saved to: $RESULTS_FILE"
echo "=================================================="
echo ""
cat "$RESULTS_FILE"
echo ""
echo "Log file: $LOG_FILE"
echo "Full analysis: $TEST_DIR/analysis.md"
