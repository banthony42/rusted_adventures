#!/bin/bash

MY_DIR="${0%/*}"

[ -r .env ] && {
    . .env
} || {
	echo ".env file not found, please provide one."
	exit 1
}

CONTAINER_NAME=${CONTAINER_NAME:-"rpg-postgres-1"}


DATABASE_URL="postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@localhost:5432/$POSTGRES_DB"

until pg_isready -d "$DATABASE_URL" >/dev/null 2>&1; do
	echo "Waiting for PostgreSQL to be ready ...";
	sleep 1;
done; 

psql -d "$DATABASE_URL" --file="${MY_DIR}/seed.sql"