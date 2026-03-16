#!/bin/bash
set -euo pipefail

# Quick syntax check
cargo check -p bluejay-parser 2>&1 | tail -5
if [ ${PIPESTATUS[0]} -ne 0 ]; then
  echo "METRIC schema_µs=0"
  echo "METRIC kitchen_sink_µs=0"
  exit 1
fi

# Run benchmarks with JSON output, extract times
OUTPUT=$(cargo bench --bench parse -- --warm-up-time 2 --measurement-time 3 2>&1)

# Extract times in µs from criterion output
SCHEMA_NS=$(echo "$OUTPUT" | grep "parse github schema definitions" -A 3 | grep "time:" | sed 's/.*\[//;s/ .*//' | head -1)
KITCHEN_NS=$(echo "$OUTPUT" | grep "parse kitchen sink executable document" -A 3 | grep "time:" | sed 's/.*\[//;s/ .*//' | head -1)

# Parse the criterion time values (they can be in ms or µs)
parse_to_us() {
  local line="$1"
  # Get the value and unit from criterion "time: [X unit Y unit Z unit]"
  local val unit
  val=$(echo "$line" | grep "time:" | sed 's/.*\[//;s/\].*//' | awk '{print $1}')
  unit=$(echo "$line" | grep "time:" | sed 's/.*\[//;s/\].*//' | awk '{print $2}')
  
  case "$unit" in
    ms) echo "$val" | awk '{printf "%.0f", $1 * 1000}';;
    µs|"µs") echo "$val" | awk '{printf "%.0f", $1}';;
    ns) echo "$val" | awk '{printf "%.0f", $1 / 1000}';;
    *) echo "$val" | awk '{printf "%.0f", $1}';;
  esac
}

SCHEMA_LINE=$(echo "$OUTPUT" | grep "parse github schema definitions" -A 3 | grep "time:")
KITCHEN_LINE=$(echo "$OUTPUT" | grep "parse kitchen sink executable document" -A 3 | grep "time:")

SCHEMA_US=$(parse_to_us "$SCHEMA_LINE")
KITCHEN_US=$(parse_to_us "$KITCHEN_LINE")

TOTAL=$((SCHEMA_US + KITCHEN_US))

echo "METRIC total_µs=$TOTAL"
echo "METRIC schema_µs=$SCHEMA_US"
echo "METRIC kitchen_sink_µs=$KITCHEN_US"
