#!/bin/bash
set -euo pipefail

# Run parser tests (the main constraint)
cargo test -p bluejay-parser 2>&1 | grep -E "(FAILED|error|test result)" | tail -20

# Also run core tests since we may touch bluejay-core
cargo test -p bluejay-core 2>&1 | grep -E "(FAILED|error|test result)" | tail -20
