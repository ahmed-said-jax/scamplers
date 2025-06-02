#!/usr/bin/env bash

set -euo pipefail

cd rust
cargo clippy --fix --workspace --allow-dirty --all-targets --exclude scamplers-schema
cargo +nightly fmt
