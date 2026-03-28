#!/usr/bin/env bash
# Chạy 01_create_database_and_roles.sql
#   export PGPASSWORD=postgres
#   ./scripts/db/run_postgres_init.sh

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SQL1="$ROOT/scripts/db/postgres/01_create_database_and_roles.sql"
SQL2="$ROOT/scripts/db/postgres/02_grant_app_after_migrations.sql"

psql -h "${PGHOST:-localhost}" -p "${PGPORT:-5432}" -U "${PGUSER:-postgres}" \
  -v ON_ERROR_STOP=1 -f "$SQL1"

echo "OK. Sau khi chạy migrate schema:"
echo "  psql -h \"\${PGHOST:-localhost}\" -p \"\${PGPORT:-5432}\" -U crm_admin -d crm -v ON_ERROR_STOP=1 -f \"$SQL2\""
echo "Hoặc: ./scripts/db/run_postgres_grant.sh"
