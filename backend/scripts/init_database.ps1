# Applies sqlx migrations to PostgreSQL (includes seed users from 018 when first run).
#
# Prerequisites:
#   - sqlx-cli: cargo install sqlx-cli --version 0.7.4 --no-default-features --features postgres,rustls
#   - Database must exist; role must own DB or have rights to create tables.
#   - If `sqlx` is not in PATH, this script uses %USERPROFILE%\.cargo\bin\sqlx.exe when present.
#
# Usage (from backend directory):
#   .\scripts\init_database.ps1
#   $env:DATABASE_URL = "postgresql://user:pass@host:5432/crm"; .\scripts\init_database.ps1

param(
    [string] $DatabaseUrl = ""
)

$ErrorActionPreference = "Stop"
$BackendRoot = Split-Path -Parent $PSScriptRoot
Set-Location $BackendRoot

function Read-DatabaseUrlFromEnvFile {
    $path = Join-Path $BackendRoot ".env"
    if (-not (Test-Path $path)) { return $null }
    foreach ($line in Get-Content $path) {
        $t = $line.Trim()
        if ($t -eq "" -or $t.StartsWith("#")) { continue }
        if ($t -match '^\s*DATABASE_URL\s*=\s*(.+)\s*$') {
            return $Matches[1].Trim().Trim('"').Trim("'")
        }
    }
    return $null
}

# Priority: -DatabaseUrl > backend/.env > $env:DATABASE_URL > default Postgres
$url = $null
if ($DatabaseUrl -ne "") {
    $url = $DatabaseUrl
} else {
    $fromFile = Read-DatabaseUrlFromEnvFile
    if ($fromFile) {
        $url = $fromFile
    } elseif ($env:DATABASE_URL) {
        $url = $env:DATABASE_URL
    } else {
        $url = "postgresql://crm_app:crm_app@127.0.0.1:5432/crm"
    }
}

if ($url -match '^\s*sqlite:') {
    Write-Host "DATABASE_URL đang là SQLite, nhưng sqlx-cli của bạn chỉ build với driver PostgreSQL." -ForegroundColor Yellow
    Write-Host "Sửa backend\.env (hoặc xóa biến môi trường DATABASE_URL kiểu sqlite trong Windows)." -ForegroundColor Yellow
    Write-Host "Ví dụ: DATABASE_URL=postgresql://USER:PASS@127.0.0.1:5432/crm"
    exit 1
}

$env:DATABASE_URL = $url
Write-Host "DATABASE_URL=$($env:DATABASE_URL)"

$sqlx = $null
if (Get-Command sqlx -ErrorAction SilentlyContinue) {
    $sqlx = "sqlx"
} elseif (Test-Path "$env:USERPROFILE\.cargo\bin\sqlx.exe") {
    $sqlx = "$env:USERPROFILE\.cargo\bin\sqlx.exe"
}

if (-not $sqlx) {
    Write-Host "sqlx CLI not found. Install:" -ForegroundColor Yellow
    Write-Host '  cargo install sqlx-cli --version 0.7.4 --no-default-features --features postgres,rustls'
    Write-Host "Then reopen the terminal or add $env:USERPROFILE\.cargo\bin to PATH."
    exit 1
}

# Optional: create database if your role has CREATEDB (ignore errors otherwise)
& $sqlx database create 2>$null

& $sqlx migrate run --source ./migrations

Write-Host "Done. Migrations applied (including system users if 018 is new)."
Write-Host "Accounts (change passwords in production):"
Write-Host "  administrator@system.local / AdministratorInit#2026"
Write-Host "  application@system.local   / ApplicationInit#2026"
