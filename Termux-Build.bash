#!/usr/bin/env bash

# Exit immediately if any command fails
set -e

# Configuration
BINARY_NAME="ejected-trail-cli"
TARGET_DIR="target/release"
INSTALL_DIR="$HOME/.local/bin"
DB_FILE="trails_cache.redb"

echo "=========================================="
echo "Ejected Media - Termux Build & Run Utility"
echo "=========================================="

# Step 1: Clean and update dependencies if requested
if [ "$1" == "--clean" ]; then
    echo "[*] Cleaning previous build artifacts..."
    cargo clean
fi

# Step 2: Run cargo check/tests to validate code integrity
echo "[*] Running cargo test suite..."
cargo test

# Step 3: Build optimized release binary for local architecture
echo "[*] Compiling release binary in Rust..."
cargo build --release

# Step 4: Ensure local bin directory exists and install binary
if [ ! -d "$INSTALL_DIR" ]; then
    echo "[*] Creating local binary directory: $INSTALL_DIR"
    mkdir -p "$INSTALL_DIR"
fi

echo "[*] Installing binary to $INSTALL_DIR/$BINARY_NAME..."
cp "$TARGET_DIR/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "=========================================="
echo "Build Successful! Binary ready for offline use."
echo "=========================================="

# Step 5: Execute quick health check command if database exists
if [ -f "$DB_FILE" ]; then
    echo "[*] Executing quick network status check..."
    "$INSTALL_DIR/$BINARY_NAME" --db "$DB_FILE" stats
else
    echo "[!] Notice: Database '$DB_FILE' not found yet. Run your initializer to populate redb."
fi
