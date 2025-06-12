#! /usr/bin/env bash

set -euo pipefail

function cleanup_docker() {
    docker kill scamplers-test > /dev/null
    docker rm scamplers-test --volumes > /dev/null
}
trap cleanup_docker EXIT

cargo test --package scamplers-backend -- --show-output
