#!/bin/bash
# Quick Start Script for Backend (Linux/Mac)

echo "========================================"
echo "   CRM Backend Quick Start"
echo "========================================"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "[ERROR] Rust/Cargo not found!"
    echo ""
    echo "Please install Rust from: https://rustup.rs/"
    echo "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    exit 1
fi

echo "[OK] Rust found:"
cargo --version
echo ""

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "[INFO] Creating .env file from .env.example..."
    cp .env.example .env
    echo "[WARN] Please edit .env file and set JWT_SECRET!"
    echo ""
fi

# Create data and uploads directories
if [ ! -d "data" ]; then
    echo "[INFO] Creating data directory..."
    mkdir -p data
fi

if [ ! -d "uploads" ]; then
    echo "[INFO] Creating uploads directory..."
    mkdir -p uploads
fi

echo "========================================"
echo "   Building and Running Backend"
echo "========================================"
echo ""

# Build
echo "[INFO] Building project (this may take a few minutes on first run)..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "[ERROR] Build failed!"
    exit 1
fi

echo ""
echo "[SUCCESS] Build completed!"
echo ""
echo "========================================"
echo "   Starting Server"
echo "========================================"
echo ""
echo "Server will start on: http://localhost:3000"
echo "Health check: http://localhost:3000/health"
echo "API docs: http://localhost:3000/api"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Run the server
cargo run --release
