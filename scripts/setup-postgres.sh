#!/usr/bin/env bash

set -euo pipefail

readonly database_host="${BEEHIVE_DB_HOST:-127.0.0.1}"
readonly database_port="${BEEHIVE_DB_PORT:-5432}"
readonly admin_database="${BEEHIVE_DB_ADMIN_DATABASE:-postgres}"
readonly admin_user="${BEEHIVE_DB_ADMIN_USER:-$(id -un)}"
readonly application_role="beehive_vault"
readonly databases=("beehive_vault" "beehive_vault_test")

if ! pg_isready --host "$database_host" --port "$database_port" >/dev/null; then
    echo "PostgreSQL is not accepting connections on ${database_host}:${database_port}." >&2
    exit 1
fi

role_exists="$(
    psql \
        --host "$database_host" \
        --port "$database_port" \
        --username "$admin_user" \
        --dbname "$admin_database" \
        --tuples-only \
        --no-align \
        --command "SELECT 1 FROM pg_roles WHERE rolname = '${application_role}'"
)"

if [[ "$role_exists" != "1" ]]; then
    psql \
        --host "$database_host" \
        --port "$database_port" \
        --username "$admin_user" \
        --dbname "$admin_database" \
        --set ON_ERROR_STOP=1 \
        --command "CREATE ROLE ${application_role} LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT"
fi

for database_name in "${databases[@]}"; do
    database_exists="$(
        psql \
            --host "$database_host" \
            --port "$database_port" \
            --username "$admin_user" \
            --dbname "$admin_database" \
            --tuples-only \
            --no-align \
            --command "SELECT 1 FROM pg_database WHERE datname = '${database_name}'"
    )"

    if [[ "$database_exists" != "1" ]]; then
        createdb \
            --host "$database_host" \
            --port "$database_port" \
            --username "$admin_user" \
            --owner "$application_role" \
            "$database_name"
    fi

    psql \
        --host "$database_host" \
        --port "$database_port" \
        --username "$admin_user" \
        --dbname "$admin_database" \
        --set ON_ERROR_STOP=1 \
        --command "ALTER DATABASE ${database_name} SET timezone TO 'UTC'" \
        >/dev/null
done

echo "PostgreSQL development databases are ready."
