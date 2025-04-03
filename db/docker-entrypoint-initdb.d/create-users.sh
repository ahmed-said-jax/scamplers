#!/bin/bash
set -eoux pipefail

root_user=$POSTGRES_USER
root_password=$POSTGRES_PASSWORD
db_name=$POSTGRES_DB
login_user_password=$(cat /run/secrets/db_login_user_password)
auth_user_password=$(cat /run/secrets/db_auth_user_password)

psql -v ON_ERROR_STOP=1 --username "$root_user" --password "$root_password" --dbname "$db_name" <<-EOSQL
	create user login_user with password '${login_user_password}';
	create user auth_user with password '${auth_user_password}';
EOSQL
