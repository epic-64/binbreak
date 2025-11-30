#!/bin/bash

# Build script for binbreak game
# Builds for Linux and Windows, copies executables to executables/ directory

set -e  # Exit on error

echo "🔨 Building binbreak for multiple platforms..."
echo ""

# Create executables directory if it doesn't exist
EXEC_DIR="executables"
mkdir -p "$EXEC_DIR"

# Get version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -n1 | cut -d'"' -f2)

# Build for Linux (native)
echo "📦 Building for Linux (x86_64)..."
cargo build --release
LINUX_BIN="binbreak-v${VERSION}-linux-x86_64"
cp "target/release/binbreak" "$EXEC_DIR/$LINUX_BIN"
echo "✅ Linux build complete: $EXEC_DIR/$LINUX_BIN"
echo ""

# Build for Windows
echo "📦 Building for Windows (x86_64)..."
if ! rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
    echo "⚠️  Installing Windows target (x86_64-pc-windows-gnu)..."
    rustup target add x86_64-pc-windows-gnu
fi

cargo build --release --target x86_64-pc-windows-gnu
WINDOWS_BIN="binbreak-v${VERSION}-windows-x86_64.exe"
cp "target/x86_64-pc-windows-gnu/release/binbreak.exe" "$EXEC_DIR/$WINDOWS_BIN"
echo "✅ Windows build complete: $EXEC_DIR/$WINDOWS_BIN"
echo ""

# Print summary
echo "🎉 All builds complete!"
echo ""
echo "Executables:"
ls -lh "$EXEC_DIR" | tail -n +2
echo ""
echo "Location: $EXEC_DIR/"

