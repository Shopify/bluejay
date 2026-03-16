#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")"

# Quick syntax check
cargo check -p bluejay-validator 2>&1 | tail -5

# Run benchmarks - extract times for all cases
OUTPUT=$(cargo bench --bench validate -- --warm-up-time 2 --measurement-time 3 --output-format bencher 2>/dev/null | grep "^test ")

# Parse results: "test name ... bench: X ns/iter (+/- Y)"
# We sum validate/{simple,fragments,complex} as the primary metric
total_ns=0
for case in simple fragments complex; do
    ns=$(echo "$OUTPUT" | grep "validate/${case}" | sed 's/.*bench: *\([0-9,]*\) ns.*/\1/' | tr -d ',')
    if [ -n "$ns" ]; then
        echo "METRIC validate_${case}_ns=$ns"
        total_ns=$((total_ns + ns))
    fi
done

echo "METRIC total_ns=$total_ns"

# Also capture field_selection_merging/128 for reference
FSM_OUTPUT=$(cargo bench --bench field_selection_merging -- --warm-up-time 1 --measurement-time 2 --output-format bencher "128" 2>/dev/null | grep "^test " || true)
fsm_ns=$(echo "$FSM_OUTPUT" | grep "128" | sed 's/.*bench: *\([0-9,]*\) ns.*/\1/' | tr -d ',' || echo "0")
if [ -n "$fsm_ns" ] && [ "$fsm_ns" != "0" ]; then
    echo "METRIC fsm_128_ns=$fsm_ns"
fi
