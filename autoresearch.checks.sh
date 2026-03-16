#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")"

# Run validator tests
cargo test -p bluejay-validator 2>&1 | tail -10

# Also run core tests since we might touch traits
cargo test -p bluejay-core 2>&1 | tail -5
