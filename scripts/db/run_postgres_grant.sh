#!/usr/bin/env bash
# Chạy 02_grant_app_after_migrations.sql sau migrate.
#   export PGPASSWORD='...'  # mật khẩu crm_admin
#   ./scripts/db/run_postgres_grant.sh

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SQL2="$ROOT/scripts/db/postgres/02_grant_app_after_migrations.sql"

psql -h "${PGHOST:-localhost}" -p "${PGPORT:-5432}" -U "${PGUSER:-crm_admin}" -d "${PGDATABASE:-crm}" \
  -v ON_ERROR_STOP=1 -f "$SQL2"

echo "OK."
