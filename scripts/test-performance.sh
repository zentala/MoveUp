#!/usr/bin/env bash

# test-performance.sh — Run tests and generate performance report
#
# Usage:
#   ./scripts/test-performance.sh
#   ./scripts/test-performance.sh --filter "session"

set -e

FILTER="${1:-.}"
OUTPUT_DIR="test-performance-report"
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)

echo "📊 Running tests with performance metrics..."
echo "Filter: $FILTER"
echo "Output: $OUTPUT_DIR/$TIMESTAMP/"

mkdir -p "$OUTPUT_DIR/$TIMESTAMP"

# Run tests with timing
echo "🧪 Running Rust tests..."
cd src-tauri

cargo test --lib "$FILTER" -- --nocapture --test-threads=1 2>&1 | tee "../$OUTPUT_DIR/$TIMESTAMP/rust-tests.log"

echo ""
echo "✅ Test run complete!"
echo ""
echo "📈 Performance Analysis:"
echo "═════════════════════════"

# Parse and display slow tests
echo ""
echo "Slowest Tests (>100ms):"
echo "─────────────────────────"
grep -E "test.*\.\.\. ok.*\(" "../$OUTPUT_DIR/$TIMESTAMP/rust-tests.log" | \
  awk '{print $NF}' | \
  sed 's/[()]//g' | \
  awk -F'ms' '{print $1 " ms\t" $0}' | \
  sort -rn | \
  awk '$1 > 100 {print $0}' | \
  head -10 || echo "  (none > 100ms ✅)"

echo ""
echo "📁 Full report: $OUTPUT_DIR/$TIMESTAMP/"
echo "   - rust-tests.log: Detailed test output"
echo ""

cd ..

# Generate HTML summary
cat > "$OUTPUT_DIR/$TIMESTAMP/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
  <title>Test Performance Report</title>
  <style>
    body { font-family: system-ui; margin: 20px; }
    h1 { color: #333; }
    .metric {
      background: #f5f5f5;
      padding: 10px;
      border-left: 4px solid #007bff;
      margin: 10px 0;
    }
    table { border-collapse: collapse; width: 100%; }
    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
    th { background-color: #f0f0f0; }
    .slow { background-color: #fff3cd; }
  </style>
</head>
<body>
  <h1>Test Performance Report</h1>
  <p>Generated: <code>TIMESTAMP_PLACEHOLDER</code></p>
  <p>To view detailed logs, see: <a href="rust-tests.log">rust-tests.log</a></p>
  <div class="metric">
    <strong>💡 Tip:</strong> Tests > 100ms should be investigated for optimization opportunities.
  </div>
</body>
</html>
EOF

sed -i "s/TIMESTAMP_PLACEHOLDER/$(date -u +%Y-%m-%dT%H:%M:%SZ)/g" "$OUTPUT_DIR/$TIMESTAMP/index.html"

echo "📍 Open in browser: file://$(pwd)/$OUTPUT_DIR/$TIMESTAMP/index.html"
