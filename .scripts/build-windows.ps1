# Windows Build Script for gh_cherry
# Builds a standalone executable for Windows

Write-Host "🔨 Building gh_cherry for Windows..." -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Cargo not found. Please install Rust: https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Clean previous builds
Write-Host "🧹 Cleaning previous builds..." -ForegroundColor Yellow
cargo clean

# Build release for Windows (current platform)
Write-Host "🚀 Building release for Windows x64..." -ForegroundColor Blue
cargo build --release --target x86_64-pc-windows-msvc

if ($LASTEXITCODE -eq 0) {
    $exe_path = "target\x86_64-pc-windows-msvc\release\gh_cherry.exe"
    $output_path = "dist\gh_cherry-windows-x64.exe"
    
    # Create dist directory
    if (!(Test-Path "dist")) {
        New-Item -ItemType Directory -Path "dist" | Out-Null
    }
    
    # Copy the executable
    Copy-Item $exe_path $output_path -Force
    
    # Get file size
    $size = (Get-Item $output_path).Length
    $sizeMB = [math]::Round($size / 1MB, 2)
    
    Write-Host "✅ Build successful!" -ForegroundColor Green
    Write-Host "📦 Output: $output_path" -ForegroundColor Cyan
    Write-Host "📏 Size: $sizeMB MB" -ForegroundColor Cyan
    
    # Optional: Strip debug symbols to reduce size
    Write-Host "🔧 Stripping debug symbols..." -ForegroundColor Yellow
    if (Get-Command "strip" -ErrorAction SilentlyContinue) {
        strip $output_path
        $newSize = (Get-Item $output_path).Length
        $newSizeMB = [math]::Round($newSize / 1MB, 2)
        Write-Host "📏 New size: $newSizeMB MB" -ForegroundColor Cyan
    } else {
        Write-Host "⚠️  Strip not available. Install mingw-w64 or use Windows Subsystem for Linux for smaller binaries." -ForegroundColor Yellow
    }
    
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "🎉 Windows build complete!" -ForegroundColor Green
