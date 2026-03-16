#!/bin/bash
set -euo pipefail

# Quick syntax check
cargo check -p bluejay-parser 2>&1 | tail -5
if [ ${PIPESTATUS[0]} -ne 0 ]; then
  echo "METRIC exec_µs=0"
  echo "METRIC schema_µs=0"
  echo "METRIC kitchen_sink_µs=0"
  exit 1
fi

# Run benchmarks
OUTPUT=$(cargo bench --bench parse -- --warm-up-time 2 --measurement-time 3 2>&1)

# Parse criterion time value to µs
parse_to_us() {
  local line="$1"
  local val unit
  val=$(echo "$line" | sed 's/.*\[//;s/\].*//' | awk '{print $1}')
  unit=$(echo "$line" | sed 's/.*\[//;s/\].*//' | awk '{print $2}')
  
  case "$unit" in
    ms) echo "$val" | awk '{printf "%.0f", $1 * 1000}';;
    µs|"µs") echo "$val" | awk '{printf "%.0f", $1}';;
    ns) echo "$val" | awk '{printf "%.2f", $1 / 1000}';;
    *) echo "$val" | awk '{printf "%.0f", $1}';;
  esac
}

SCHEMA_LINE=$(echo "$OUTPUT" | grep "parse github schema definitions" -A 3 | grep "time:")
KITCHEN_LINE=$(echo "$OUTPUT" | grep "parse kitchen sink executable document" -A 3 | grep "time:")
EXEC_LINE=$(echo "$OUTPUT" | grep "parse large executable document" -A 3 | grep "time:")

SCHEMA_US=$(parse_to_us "$SCHEMA_LINE")
KITCHEN_US=$(parse_to_us "$KITCHEN_LINE")
EXEC_US=$(parse_to_us "$EXEC_LINE")

echo "METRIC exec_µs=$EXEC_US"
echo "METRIC schema_µs=$SCHEMA_US"
echo "METRIC kitchen_sink_µs=$KITCHEN_US"
