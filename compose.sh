#! /usr/bin/env bash

set -euxo pipefail

cd rust/build-scamplers && cargo run --release
docker compose up --build --detach
