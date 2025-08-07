#!/bin/bash
# macOS Build Script for gh_cherry
# Builds a standalone executable for macOS

set -e

echo "🔨 Building gh_cherry for macOS..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Install target if needed
echo "📥 Ensuring macOS targets are installed..."
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for Intel Macs
echo "🚀 Building for Intel Macs (x86_64)..."
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon Macs
echo "🚀 Building for Apple Silicon (ARM64)..."
cargo build --release --target aarch64-apple-darwin

# Create dist directory
mkdir -p dist

# Copy binaries
cp target/x86_64-apple-darwin/release/gh_cherry dist/gh_cherry-macos-intel
cp target/aarch64-apple-darwin/release/gh_cherry dist/gh_cherry-macos-arm64

# Create universal binary
echo "🔄 Creating universal binary..."
lipo -create -output dist/gh_cherry-macos-universal \
    dist/gh_cherry-macos-intel \
    dist/gh_cherry-macos-arm64

# Strip debug symbols
echo "🔧 Stripping debug symbols..."
strip dist/gh_cherry-macos-intel
strip dist/gh_cherry-macos-arm64  
strip dist/gh_cherry-macos-universal

# Get file sizes
intel_size=$(du -h dist/gh_cherry-macos-intel | cut -f1)
arm64_size=$(du -h dist/gh_cherry-macos-arm64 | cut -f1)
universal_size=$(du -h dist/gh_cherry-macos-universal | cut -f1)

echo "✅ Build successful!"
echo "📦 Intel binary: dist/gh_cherry-macos-intel ($intel_size)"
echo "📦 ARM64 binary: dist/gh_cherry-macos-arm64 ($arm64_size)"
echo "📦 Universal binary: dist/gh_cherry-macos-universal ($universal_size)"

# Make executables
chmod +x dist/gh_cherry-macos-*

echo "🎉 macOS build complete!"

# Optional: Create .app bundle
read -p "🍎 Create .app bundle? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📱 Creating .app bundle..."
    
    # Create app structure
    mkdir -p "dist/gh_cherry.app/Contents/MacOS"
    mkdir -p "dist/gh_cherry.app/Contents/Resources"
    
    # Copy binary
    cp dist/gh_cherry-macos-universal "dist/gh_cherry.app/Contents/MacOS/gh_cherry"
    
    # Create Info.plist
    cat > "dist/gh_cherry.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>gh_cherry</string>
    <key>CFBundleIdentifier</key>
    <string>com.github.gh_cherry</string>
    <key>CFBundleName</key>
    <string>GitHub Cherry-Pick TUI</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF
    
    echo "📱 .app bundle created: dist/gh_cherry.app"
fi
