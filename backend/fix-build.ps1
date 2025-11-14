# Script to fix "Access is denied" build error
# Run this script if cargo build fails with "Access is denied"

Write-Host "🔧 Fixing build issues..." -ForegroundColor Yellow

# 1. Kill any running crm-backend processes
Write-Host "`n1. Checking for running processes..." -ForegroundColor Cyan
$processes = Get-Process | Where-Object {$_.ProcessName -like "*crm-backend*"}
if ($processes) {
    Write-Host "   Found running processes, stopping them..." -ForegroundColor Yellow
    $processes | Stop-Process -Force
    Start-Sleep -Seconds 2
} else {
    Write-Host "   No running processes found ✓" -ForegroundColor Green
}

# 2. Try to remove the locked .exe file
Write-Host "`n2. Removing locked .exe file..." -ForegroundColor Cyan
$exePath = "target\debug\crm-backend.exe"
if (Test-Path $exePath) {
    try {
        Remove-Item -Path $exePath -Force -ErrorAction Stop
        Write-Host "   File removed successfully ✓" -ForegroundColor Green
    } catch {
        Write-Host "   Warning: Could not remove file: $_" -ForegroundColor Yellow
        Write-Host "   You may need to close your IDE or restart your computer" -ForegroundColor Yellow
    }
} else {
    Write-Host "   File not found (already removed) ✓" -ForegroundColor Green
}

# 3. Clean and rebuild
Write-Host "`n3. Cleaning build artifacts..." -ForegroundColor Cyan
cargo clean

Write-Host "`n4. Building project..." -ForegroundColor Cyan
cargo build

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ Build successful!" -ForegroundColor Green
} else {
    Write-Host "`n❌ Build failed. Try:" -ForegroundColor Red
    Write-Host "   - Close your IDE completely" -ForegroundColor Yellow
    Write-Host "   - Restart your computer" -ForegroundColor Yellow
    Write-Host "   - Check Windows Defender exclusions" -ForegroundColor Yellow
}

