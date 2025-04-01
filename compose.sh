#! /usr/bin/env bash

set -euxo pipefail

cd rust && cargo run --release --bin build-scamplers
docker compose up --build --detach
