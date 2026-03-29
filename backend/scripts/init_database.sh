#!/usr/bin/env bash
# Applies sqlx migrations to PostgreSQL (includes seed users from 018 when first run).
#
# Prerequisites:
#   - sqlx-cli: cargo install sqlx-cli --version 0.7.4 --no-default-features --features postgres,rustls
#   - Database must exist. Default URL uses crm_app: that user needs USAGE+CREATE on schema public
#     (see scripts/db/postgres/01_create_database_and_roles.sql). Alternatively run migrate as crm_admin.
#
# Usage:
#   cd backend && chmod +x scripts/init_database.sh && ./scripts/init_database.sh
#   DATABASE_URL=postgresql://crm_app:crm_app@127.0.0.1:5432/crm ./scripts/init_database.sh

set -euo pipefail
BACKEND_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$BACKEND_ROOT"

# Prefer DATABASE_URL from backend/.env if set (avoids stale sqlite: in environment)
if [[ -f "$BACKEND_ROOT/.env" ]]; then
  line="$(grep -E '^[[:space:]]*DATABASE_URL=' "$BACKEND_ROOT/.env" | head -1 || true)"
  if [[ -n "$line" ]]; then
    export DATABASE_URL="${line#*=}"
    DATABASE_URL="${DATABASE_URL%\"}"
    DATABASE_URL="${DATABASE_URL#\"}"tich
    
    DATABASE_URL="${DATABASE_URL%\'}"
    DATABASE_URL="${DATABASE_URL#\'}"
  fi
fi
export DATABASE_URL="${DATABASE_URL:-postgresql://crm_app:crm_app@127.0.0.1:5432/crm}"

if [[ "$DATABASE_URL" == sqlite:* ]]; then
  echo "DATABASE_URL is SQLite but sqlx-cli was built with PostgreSQL only." >&2
  echo "Fix backend/.env or unset DATABASE_URL (use postgresql://...)." >&2
  exit 1
fi

echo "DATABASE_URL=$DATABASE_URL"

SQLX="sqlx"
if ! command -v sqlx >/dev/null 2>&1; then
  if [[ -x "${HOME}/.cargo/bin/sqlx" ]]; then
    SQLX="${HOME}/.cargo/bin/sqlx"
  else
    echo "sqlx CLI not found. Install:" >&2
    echo "  cargo install sqlx-cli --version 0.7.4 --no-default-features --features postgres,rustls" >&2
    exit 1
  fi
fi

"$SQLX" database create 2>/dev/null || true
"$SQLX" migrate run --source ./migrations

echo "Done. Migrations applied (including system users if 018 is new)."
echo "Accounts (change passwords in production):"
echo "  administrator@system.local / AdministratorInit#2026"
echo "  application@system.local   / ApplicationInit#2026"
