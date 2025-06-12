#! /usr/bin/env bash

set -euo pipefail

function cleanup_docker() {
    docker kill scamplers-backend_unit_test > /dev/null
    docker rm scamplers-backend_unit_test --volumes > /dev/null
}
trap cleanup_docker EXIT

cargo test --package scamplers-backend
