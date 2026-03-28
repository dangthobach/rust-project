# Chạy 02_grant_app_after_migrations.sql (sau khi schema đã có bảng).
# Yêu cầu: PGPASSWORD cho crm_admin hoặc .pgpass

param(
    [string] $PgHost = "localhost",
    [string] $PgUser = "crm_admin",
    [string] $PgPort = "5432",
    [string] $Database = "crm"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$sql2 = Join-Path $RepoRoot "scripts\db\postgres\02_grant_app_after_migrations.sql"

if (-not (Test-Path $sql2)) {
    Write-Error "Missing $sql2"
}

Write-Host "Running $sql2 as ${PgUser}@${PgHost}/${Database} ..."
& psql -h $PgHost -p $PgPort -U $PgUser -d $Database -v ON_ERROR_STOP=1 -f $sql2
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "OK."
