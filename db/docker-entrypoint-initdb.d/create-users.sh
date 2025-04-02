#!/bin/bash
set -e

login_user_password=$(cat /run/secrets/db_login_user_password)
auth_user_password=$(cat /run/secrets/db_auth_user_password)

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --password "$POSTGRES_PASSWORD" --dbname "$POSTGRES_DB" <<-EOSQL
	create login_user with password '$login_user_password';
	create auth_user with password '$auth_user_password';
EOSQL
