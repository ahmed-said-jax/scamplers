#!/usr/bin/env bash

set -e

source .env

# Handle CTRL-C
cleanup() {
  echo "Cleaning up..."
  kill "$RUST_PID" "$PYTHON_PID" 2>/dev/null || true
  docker stop "$DB_CONTAINER" 2>/dev/null || true
  exit
}
trap cleanup INT

# Start database
echo "Starting database..."
DB_CONTAINER=$(docker run \
  --rm \
  --detach \
  --env POSTGRES_USER="$SCAMPLERS_DB_ROOT_USER" \
  --env POSTGRES_PASSWORD="$SCAMPLERS_DB_ROOT_PASSWORD" \
  --publish 5432:"$SCAMPLERS_DB_PORT" \
  postgres:"$SCAMPLERS_POSTGRES_VERSION")
echo "Database container ID: $DB_CONTAINER"

# Start Rust server after frontend build
(
  cd typescript/scamplers-web
  echo "Building frontend..."
  npm run build
  echo "Frontend built."
  cd ../../rust
  echo "Starting Rust server..."
  cargo run -- test
) &
RUST_PID=$!

# Wait for Rust server to be healthy
HEALTH_URL="http://${SCAMPLERS_APP_HOST}:${SCAMPLERS_APP_PORT}/health"
echo "Waiting for Rust server to become healthy at $HEALTH_URL..."
until curl --silent --fail-with-body "$HEALTH_URL" >/dev/null; do
  echo "Rust server not up yet. Retrying in 5s..."
  sleep 5
done
echo "Rust server is up!"

# Start Python server
(
  cd python/scamplers-auth
  echo "Starting Python server..."
  uv run main.py
) &
PYTHON_PID=$!

# Wait for both to exit (or for CTRL-C)
wait "$RUST_PID" "$PYTHON_PID"
