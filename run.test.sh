#! /usr/bin/env bash

set -euxo pipefail

source .env

# Handle CTRL-C
cleanup() {
  echo "stopping..."
  docker stop "$DB_CONTAINER" 2>/dev/null || true
  kill "$RUST_PID" "$PYTHON_PID" 2>/dev/null || true
  exit
}
trap cleanup INT

# Start database
echo "starting postgres..."
DB_CONTAINER=$(docker run \
  --rm \
  --detach \
  --env POSTGRES_USER="$SCAMPLERS_DB_ROOT_USER" \
  --env POSTGRES_PASSWORD="$SCAMPLERS_DB_ROOT_PASSWORD" \
  --env POSTGRES_DB="$SCAMPLERS_DB_NAME" \
  --publish 5432:"$SCAMPLERS_DB_PORT" \
  postgres:"$SCAMPLERS_POSTGRES_VERSION")
echo $DB_CONTAINER

# Start Rust server after frontend build
(
  cd typescript/scamplers-web
  echo "building frontend..."
  npm run build
  echo "built frontend"
  cd ../../rust
  echo "starting scamplers..."
  cargo run -- test
) &
RUST_PID=$!


# Wait for Rust server to be healthy
HEALTH_URL="http://${SCAMPLERS_APP_HOST}:${SCAMPLERS_APP_PORT}/health"
echo "waiting for Rust server to become healthy at $HEALTH_URL..."
until curl --silent --fail-with-body "$HEALTH_URL" >/dev/null; do
  echo "rust server not up yet. Retrying in 5s..."
  sleep 5
done
echo "rust server started"

# Start Python server
(
  cd python/scamplers-auth
  echo "running scamplers-auth"
  uv run main.py --debug=True
) &
PYTHON_PID=$!

# Wait for both to exit (or for CTRL-C)
wait "$RUST_PID" "$PYTHON_PID"
