#! /usr/bin/env bash

set -euxo pipefail

cd rust
cargo run --release --package build-scamplers
docker compose up --build --detach
