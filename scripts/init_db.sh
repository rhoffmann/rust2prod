#!/usr/bin/env bash
set -x
set -eo pipefail

# check if dependencies are installed

if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql not installed."
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 " cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
    echo >&2 "to install it."
    exit 1
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD=${POSTGRES_PASSWORD:=password}
DB_NAME=${POSTGRES_DB:=newsletter}
DB_PORT=${POSTGRES_PORT:=5432}


if [[ -z "${SKIP_DOCKER}" ]]
then
    docker run \
        -e POSTGRES_USER=${DB_USER}\
        -e POSTGRES_PASSWORD=${DB_PASSWORD}\
        -e POSTGRES_DB=${DB_NAME}\
        -p "${DB_PORT}":5432\
        -d postgres \
        postgres -N 1000
        # number of connections for testing
fi

# ping postgres until service is ready

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres unavailable -- waiting"
    sleep 1
done

>&2 echo "Postgres is running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}

sqlx database create
sqlx migrate run

>&2 echo "Postgres Database migrated, ready to go!"