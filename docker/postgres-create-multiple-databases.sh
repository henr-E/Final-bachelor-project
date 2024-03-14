#! /usr/bin/env bash

# This script creates multiple databases in a single postgresql instance.
#
# Databases should be specified in the `POSTGRES_DATABASES` environment
# variable and should be of the form:
#   `db1:password1,db2:password2,...`
# For every database a new user will be created with the specified password
# and the database name as the username.
#
# Adapted from https://github.com/MartinKaburu/docker-postgresql-multiple-databases

# Exit the script on first error.
set -e
set -u

function create_database() {
    local db_and_password="$1"
    # Replace `:` with a space and read the resulting string into the variables.
    read -r database password <<< "${db_and_password//:/ }"

    echo "## Creating database \"$database\" for owner \"$database\""
    psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<- EOSQL
        CREATE USER $database ENCRYPTED PASSWORD '$password';
        CREATE DATABASE $database;
        ALTER DATABASE $database OWNER TO $database;
        GRANT ALL PRIVILEGES ON DATABASE $database TO $database;
EOSQL
    # cannot be indented because bash :< This comment can also not be placed on
    # the previous line because of the same reason :(
}

if [ -n "$POSTGRES_DATABASES" ]; then
    echo "# Creating multiple databases"

    for db in $(echo "$POSTGRES_DATABASES" | tr ',' ' '); do
        create_database $db
    done
fi
