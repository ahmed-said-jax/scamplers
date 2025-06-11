#! /usr/bin/env bash

set -euo pipefail

function cleanup_docker() {
    docker kill scamplers-test
    docker rm scamplers-test --volumes
}

docker run --name scamplers-test --env POSTGRES_HOST_AUTH_METHOD=trust --env POSTGRES_DB=scamplers-test --publish 5432:5432 --detach postgres:18beta1-alpine

export SCAMPLERS_TEST_DB_URL="postgres://postgres@localhost:5432/scamplers-test"
export DATABASE_URL="$SCAMPLERS_TEST_DB_URL"

sleep 1

diesel migration run --migration-dir ../db/migrations

cargo test --package scamplers-backend -- --show-output

trap cleanup_docker EXIT
