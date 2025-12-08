#!/usr/bin/env bash
set -euo pipefail

SQL_INSTANCE=${SQL_INSTANCE:-embeddings-sql-europe-west2}
DB_NAME=${DB_NAME:-movies}

echo "Applying pgvector extension as postgres..."
gcloud sql connect "$SQL_INSTANCE" --user=postgres --database="$DB_NAME" --quiet < db/pgvector.sql

echo "Applying schema as postgres..."
gcloud sql connect "$SQL_INSTANCE" --user=postgres --database="$DB_NAME" --quiet < db/schema.sql

echo "Schema applied."

