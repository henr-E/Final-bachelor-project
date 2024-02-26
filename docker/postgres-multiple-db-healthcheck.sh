#! /usr/bin/env bash

if [ -z "$POSTGRES_DATABASES" ]; then
    exit 0;
fi

read -r -a databases <<< "$(echo "$POSTGRES_DATABASES" | tr ',' ' ')"

for database_and_password in "${databases[@]}"; do
    # Evil bash regex magic.
    local database="${database_and_password%%:*}"
    # Check if the database is up.
    su -c 'pg_isready -U "$database" -d "$database"' - postgres || exit 1
done
