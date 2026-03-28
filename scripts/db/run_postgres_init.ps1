# Chạy script PostgreSQL tạo crm_admin + crm_app + database crm.
# Yêu cầu: psql, quyền superuser (thường user postgres).
#
#   $env:PGPASSWORD = "postgres"; .\scripts\db\run_postgres_init.ps1

param(
    [string] $PgHost = "localhost",
    [string] $PgUser = "postgres",
    [string] $PgPort = "5432"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$sql1 = Join-Path $RepoRoot "scripts\db\postgres\01_create_database_and_roles.sql"
$sql2 = Join-Path $RepoRoot "scripts\db\postgres\02_grant_app_after_migrations.sql"

if (-not (Test-Path $sql1)) {
    Write-Error "Missing $sql1"
}

Write-Host "Running $sql1 as $PgUser@$PgHost ..."
& psql -h $PgHost -p $PgPort -U $PgUser -v ON_ERROR_STOP=1 -f $sql1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "OK. Sau khi chay migrate schema (vd: sqlx migrate -U crm_admin -d crm), chay:"
Write-Host "  psql -h $PgHost -p $PgPort -U crm_admin -d crm -v ON_ERROR_STOP=1 -f `"$sql2`""
Write-Host "Hoac: .\scripts\db\run_postgres_grant.ps1 -PgHost $PgHost -PgPort $PgPort"
