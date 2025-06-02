#!/usr/bin/env bash

set -euo pipefail

cd rust
cargo clippy --workspace --all-targets --exclude scamplers-schema
cargo +nightly fmt
