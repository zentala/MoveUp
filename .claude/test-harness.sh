#!/bin/bash
# Overlay Testing Harness
# Automatically manages: log cleanup, app launch, screenshot capture, log analysis

set -e

TEST_DATE=$(date +"%Y-%m-%d_%H-%M-%S")
TEST_DIR=".claude/test-runs/${TEST_DATE}"
LOG_FILE="${TEST_DIR}/overlay.log"
SCREENSHOT_DIR="${TEST_DIR}/screenshots"
ANALYSIS_FILE="${TEST_DIR}/analysis.md"

echo "🧪 Overlay Testing Harness"
echo "Test: $TEST_DATE"
echo ""

# Step 1: Create clean test directory
echo "📁 Creating test directory..."
mkdir -p "${TEST_DIR}"
mkdir -p "${SCREENSHOT_DIR}"

# Step 2: Kill any existing tauri process
echo "🛑 Cleaning up old processes..."
pkill -f "tauri:dev" || true
sleep 2

# Step 3: Start app with demo mode + logging
echo "🚀 Starting overlay in DEMO mode..."
echo "Command: OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true RUST_LOG=warn pnpm tauri:dev"
echo "Logs: $LOG_FILE"
echo ""

OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true RUST_LOG=warn pnpm tauri:dev > "${LOG_FILE}" 2>&1 &
APP_PID=$!
echo "App PID: $APP_PID"

# Step 4: Run for specified duration
DURATION=${1:-15}
echo "⏱️ Running for ${DURATION} seconds..."
echo ""

# Progress indicator
for i in $(seq 1 $DURATION); do
    echo -n "."
    sleep 1
done
echo ""
echo ""

# Step 5: Stop app
echo "⏹️ Stopping app..."
kill $APP_PID || true
sleep 2

# Step 6: Analyze logs
echo "📊 Analyzing logs..."
echo ""

cat > "${ANALYSIS_FILE}" << 'EOF'
# Overlay Test Analysis

## Test Info
EOF

echo "**Test Date:** $(date)" >> "${ANALYSIS_FILE}"
echo "**Duration:** ${DURATION}s" >> "${ANALYSIS_FILE}"
echo "**Log File:** $LOG_FILE" >> "${ANALYSIS_FILE}"
echo "" >> "${ANALYSIS_FILE}"

echo "## Captured Logs Summary" >> "${ANALYSIS_FILE}"
echo "" >> "${ANALYSIS_FILE}"

# Count log types
TIMER_COUNT=$(grep -c "\[TIMER\]" "${LOG_FILE}" || echo "0")
DEMO_OVERRIDE_COUNT=$(grep -c "\[DEMO-OVERRIDE\]" "${LOG_FILE}" || echo "0")
DRAW_COUNT=$(grep -c "\[DRAW\]" "${LOG_FILE}" || echo "0")

echo "- ⏱️ TIMER logs: $TIMER_COUNT" >> "${ANALYSIS_FILE}"
echo "- 🔵 DEMO-OVERRIDE logs: $DEMO_OVERRIDE_COUNT" >> "${ANALYSIS_FILE}"
echo "- 📊 DRAW logs: $DRAW_COUNT" >> "${ANALYSIS_FILE}"
echo "" >> "${ANALYSIS_FILE}"

# Extract key information
echo "## First 10 TIMER entries" >> "${ANALYSIS_FILE}"
echo '```' >> "${ANALYSIS_FILE}"
grep "\[TIMER\]" "${LOG_FILE}" | head -10 >> "${ANALYSIS_FILE}" || echo "No TIMER logs found" >> "${ANALYSIS_FILE}"
echo '```' >> "${ANALYSIS_FILE}"
echo "" >> "${ANALYSIS_FILE}"

echo "## Sample DEMO-OVERRIDE progression (every 100th entry)" >> "${ANALYSIS_FILE}"
echo '```' >> "${ANALYSIS_FILE}"
grep "\[DEMO-OVERRIDE\]" "${LOG_FILE}" | awk 'NR % 100 == 0' | head -20 >> "${ANALYSIS_FILE}" || echo "No DEMO-OVERRIDE logs found" >> "${ANALYSIS_FILE}"
echo '```' >> "${ANALYSIS_FILE}"
echo "" >> "${ANALYSIS_FILE}"

echo "## Questions: Do you see this?" >> "${ANALYSIS_FILE}"
echo "" >> "${ANALYSIS_FILE}"
echo "- [ ] demo_mode=true in logs?" >> "${ANALYSIS_FILE}"
echo "- [ ] frame_count incrementing (60, 120, 180...)?" >> "${ANALYSIS_FILE}"
echo "- [ ] stage cycling (0 → 1 → 2 → 3 → 4)?" >> "${ANALYSIS_FILE}"
echo "- [ ] progress changing (0.00% → 25.00% → 50.00% → 75.00% → 100.00%)?" >> "${ANALYSIS_FILE}"
echo "- [ ] bar_width growing?" >> "${ANALYSIS_FILE}"
echo "" >> "${ANALYSIS_FILE}"

# Step 7: Print summary
echo "✅ Test Complete"
echo ""
echo "📋 Full analysis: $ANALYSIS_FILE"
echo ""
echo "Files created:"
echo "  - $LOG_FILE (raw logs)"
echo "  - $ANALYSIS_FILE (analysis + questions)"
echo "  - $SCREENSHOT_DIR/ (for screenshots)"
echo ""
echo "Next: Check $ANALYSIS_FILE and answer the questions"
