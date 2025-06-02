#!/usr/bin/env bash

set -euo pipefail

cd rust
cargo clippy --workspace --exclude scamplers-schema
cargo +nightly fmt
