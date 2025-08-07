#!/bin/bash
# Cross-platform build script for gh_cherry
# Builds for both Windows and macOS from any platform

set -e

echo "🌍 Cross-platform build for gh_cherry"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Create dist directory
mkdir -p dist

echo "📥 Installing cross-compilation targets..."

# Install Windows target
rustup target add x86_64-pc-windows-gnu

# Install macOS targets  
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

echo "🚀 Building for all platforms..."

# Build for Windows
echo "🪟 Building for Windows..."
if cargo build --release --target x86_64-pc-windows-gnu; then
    cp target/x86_64-pc-windows-gnu/release/gh_cherry.exe dist/gh_cherry-windows-x64.exe
    echo "✅ Windows build successful"
else
    echo "⚠️  Windows build failed (might need mingw-w64 installed)"
fi

# Build for macOS Intel (only works on macOS)
echo "🍎 Building for macOS Intel..."
if rustup target list --installed | grep -q x86_64-apple-darwin; then
    if cargo build --release --target x86_64-apple-darwin 2>/dev/null; then
        cp target/x86_64-apple-darwin/release/gh_cherry dist/gh_cherry-macos-intel
        echo "✅ macOS Intel build successful"
    else
        echo "⚠️  macOS Intel build failed (requires macOS or cross-compilation tools)"
    fi
else
    echo "⚠️  macOS Intel target not available"
fi

# Build for macOS ARM64 (only works on macOS)
echo "🍎 Building for macOS ARM64..."
if rustup target list --installed | grep -q aarch64-apple-darwin; then
    if cargo build --release --target aarch64-apple-darwin 2>/dev/null; then
        cp target/aarch64-apple-darwin/release/gh_cherry dist/gh_cherry-macos-arm64
        echo "✅ macOS ARM64 build successful"
    else
        echo "⚠️  macOS ARM64 build failed (requires macOS or cross-compilation tools)"
    fi
else
    echo "⚠️  macOS ARM64 target not available"
fi

# Create universal macOS binary if both exist
if [[ -f "dist/gh_cherry-macos-intel" && -f "dist/gh_cherry-macos-arm64" ]]; then
    echo "🔄 Creating universal macOS binary..."
    if command -v lipo &> /dev/null; then
        lipo -create -output dist/gh_cherry-macos-universal \
            dist/gh_cherry-macos-intel \
            dist/gh_cherry-macos-arm64
        echo "✅ Universal macOS binary created"
    else
        echo "⚠️  lipo not available, skipping universal binary"
    fi
fi

# Strip binaries if strip is available
if command -v strip &> /dev/null; then
    echo "🔧 Stripping debug symbols..."
    for binary in dist/gh_cherry-*; do
        if [[ -f "$binary" && "$binary" != *.exe ]]; then
            strip "$binary" 2>/dev/null || true
        fi
    done
fi

# Show results
echo ""
echo "📦 Build Results:"
for file in dist/gh_cherry-*; do
    if [[ -f "$file" ]]; then
        size=$(du -h "$file" | cut -f1)
        echo "  • $(basename "$file"): $size"
    fi
done

echo ""
echo "🎉 Cross-platform build complete!"
echo ""
echo "💡 Usage:"
echo "  Windows: ./dist/gh_cherry-windows-x64.exe"
echo "  macOS:   ./dist/gh_cherry-macos-universal"
