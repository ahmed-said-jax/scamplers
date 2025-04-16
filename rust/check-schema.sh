#! /usr/bin/env bash

set -euo pipefail

DB_ROOT_USER=$(cat /run/secrets/db_root_user)
DB_ROOT_PASSWORD=$(cat /run/secrets/db_root_password)
DB_HOST=$(cat /run/secrets/db_host)
DB_PORT=$(cat /run/secrets/db_port)
DB_NAME=$(cat /run/secrets/db_name)

DB_URL="postgres://$DB_ROOT_USER:$DB_ROOT_PASSWORD@$DB_HOST:$DB_PORT/$RANDOM"

cd /app/scamplers-schema
diesel setup --database-url $DB_URL --migration-dir ../db/migrations --locked-schema
diesel drop --database-url $DB_URL
