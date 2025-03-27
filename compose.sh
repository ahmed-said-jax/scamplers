#! /usr/bin/env bash

set -euxo pipefail

cd rust/scamplers && cargo build --locked
docker compose up --build --detach
