#!/usr/bin/env sh

set -euo pipefail
cargo run -- --dev --seed-data-path ../seed_data.dev.json
