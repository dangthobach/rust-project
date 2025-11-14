@echo off
REM Quick Start Script for Backend (Windows)

echo ========================================
echo    CRM Backend Quick Start
echo ========================================
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust/Cargo not found!
    echo.
    echo Please install Rust from: https://rustup.rs/
    echo Or run: winget install Rustlang.Rustup
    echo.
    pause
    exit /b 1
)

echo [OK] Rust found:
cargo --version
echo.

REM Check if .env file exists
if not exist ".env" (
    echo [INFO] Creating .env file from .env.example...
    copy .env.example .env >nul
    echo [WARN] Please edit .env file and set JWT_SECRET!
    echo.
)

REM Create data and uploads directories
if not exist "data" (
    echo [INFO] Creating data directory...
    mkdir data
)

if not exist "uploads" (
    echo [INFO] Creating uploads directory...
    mkdir uploads
)

echo ========================================
echo    Building and Running Backend
echo ========================================
echo.

REM Build and run
echo [INFO] Building project (this may take a few minutes on first run)...
cargo build --release

if %errorlevel% neq 0 (
    echo [ERROR] Build failed!
    pause
    exit /b 1
)

echo.
echo [SUCCESS] Build completed!
echo.
echo ========================================
echo    Starting Server
echo ========================================
echo.
echo Server will start on: http://localhost:3000
echo Health check: http://localhost:3000/health
echo API docs: http://localhost:3000/api
echo.
echo Press Ctrl+C to stop the server
echo.

REM Run the server
cargo run --release

pause
