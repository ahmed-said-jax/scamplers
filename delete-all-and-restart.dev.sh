#! /usr/bin/env bash

set -euo pipefail

docker compose kill
docker compose rm
docker volume ls --format json | jq '.[].Name' --slurp | xargs docker volume rm
COMPOSE_BAKE=true docker compose --env-file ../.env.dev up --build
