#!/bin/bash

# FM Goal Musics - Cross-Platform Build Script
# Automatically detects the platform and builds appropriate package

set -e

echo "🚀 FM Goal Musics - Cross-Platform Build Script"
echo "=============================================="

# Detect operating system
OS="$(uname -s)"
case "${OS}" in
    Darwin*)
        echo "🍎 Detected macOS"
        PLATFORM="macos"
        ;;
    Linux*)
        echo "🐧 Detected Linux"
        PLATFORM="linux"
        ;;
    CYGWIN*|MINGW*|MSYS*)
        echo "🪟 Detected Windows"
        PLATFORM="windows"
        ;;
    *)
        echo "❌ Unsupported operating system: ${OS}"
        exit 1
        ;;
esac

echo "🔨 Building for platform: ${PLATFORM}"

# Common configuration
APP_NAME="FM Goal Musics"
BINARY_NAME="fm-goal-musics-gui"
VERSION="1.0.0"

# Function to build for macOS
build_macos() {
    echo "🍎 Building macOS package..."
    
    # Check if build script exists and is executable
    if [ -f "build_macos.sh" ]; then
        chmod +x build_macos.sh
        ./build_macos.sh
    else
        echo "❌ build_macos.sh not found!"
        exit 1
    fi
}

# Function to build for Windows
build_windows() {
    echo "🪟 Building Windows package..."
    
    # On Windows, we should be running the .bat file
    if [ -f "build_windows.bat" ]; then
        if command -v cmd.exe &> /dev/null; then
            cmd.exe /c build_windows.bat
        else
            echo "⚠️  Running on Unix-like system but building for Windows..."
            echo "You need to build on Windows or use cross-compilation"
            
            # Try cross-compilation if toolchain is available
            if rustup target list --installed | grep -q "x86_64-pc-windows-msvc"; then
                echo "🔨 Cross-compiling for Windows..."
                cargo build --release --bin "$BINARY_NAME" --target x86_64-pc-windows-msvc
                
                # Create distribution directory
                BUILD_DIR="build/windows-cross"
                mkdir -p "$BUILD_DIR"
                
                # Copy binary
                cp "target/x86_64-pc-windows-msvc/release/$BINARY_NAME.exe" "$BUILD_DIR/" 2>/dev/null || \
                cp "target/x86_64-pc-windows-gnu/release/$BINARY_NAME.exe" "$BUILD_DIR/" 2>/dev/null || {
                    echo "❌ Cross-compiled binary not found!"
                    exit 1
                }
                
                echo "✅ Windows cross-compilation completed!"
                echo "📍 Binary location: $BUILD_DIR/$BINARY_NAME.exe"
            else
                echo "❌ Windows toolchain not installed!"
                echo "Run: rustup target add x86_64-pc-windows-msvc"
                exit 1
            fi
        fi
    else
        echo "❌ build_windows.bat not found!"
        exit 1
    fi
}

# Function to build for Linux
build_linux() {
    echo "🐧 Building Linux package..."
    
    # Build release binary
    cargo build --release --bin "$BINARY_NAME"
    
    # Create distribution directory
    BUILD_DIR="build/linux"
    mkdir -p "$BUILD_DIR"
    
    # Copy binary
    cp "target/release/$BINARY_NAME" "$BUILD_DIR/"
    
    # Copy assets
    if [ -d "assets" ]; then
        cp -r assets "$BUILD_DIR/"
    fi
    
    # Copy default ambiance sound
    if [ -f "goal_crowd_cheer.wav" ]; then
        echo "🎵 Copying default ambiance sound..."
        cp "goal_crowd_cheer.wav" "$BUILD_DIR/"
    fi
    
    # Copy Tesseract OCR files for self-contained distribution
    if [ -d "assets/tesseract" ]; then
        echo "🔤 Copying Tesseract OCR files..."
        if [ -f "assets/tesseract/tesseract.dll" ]; then
            cp "assets/tesseract/tesseract.dll" "$BUILD_DIR/"
        fi
        if [ -f "assets/tesseract/liblept176.dll" ]; then
            cp "assets/tesseract/liblept176.dll" "$BUILD_DIR/"
        fi
        if [ -d "assets/tesseract/tessdata" ]; then
            cp -r "assets/tesseract/tessdata" "$BUILD_DIR/"
        fi
    else
        echo "⚠️  Tesseract files not found - users will need to install Tesseract OCR manually"
    fi
    
    # Create AppImage directory structure
    APPDIR="$BUILD_DIR/$APP_NAME.AppDir"
    mkdir -p "$APPDIR/usr/bin"
    mkdir -p "$APPDIR/usr/share/applications"
    mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
    
    # Copy binary to AppDir
    cp "target/release/$BINARY_NAME" "$APPDIR/usr/bin/"
    
    # Create desktop file
    cat > "$APPDIR/usr/share/applications/$BINARY_NAME.desktop" << EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Goal celebration music player for Football Manager
Exec=$BINARY_NAME
Icon=$BINARY_NAME
Categories=Game;Sports;
EOF
    
    # Create AppRun script
    cat > "$APPDIR/AppRun" << EOF
#!/bin/bash
HERE="\$(dirname "\$(readlink -f "\${0}")")"
export PATH="\${HERE}/usr/bin/:\${PATH}"
exec "\${HERE}/usr/bin/$BINARY_NAME" "\$@"
EOF
    chmod +x "$APPDIR/AppRun"
    
    # Create tar archive
    cd build
    tar -czf "linux-$APP_NAME-$(date +%Y%m%d).tar.gz" linux/
    cd ..
    
    echo "✅ Linux build completed!"
    echo "📍 Build directory: $BUILD_DIR"
    echo "📦 Tar archive: build/linux-$APP_NAME-$(date +%Y%m%d).tar.gz"
}

# Main build logic
case "${PLATFORM}" in
    macos)
        build_macos
        ;;
    windows)
        build_windows
        ;;
    linux)
        build_linux
        ;;
    *)
        echo "❌ Unsupported platform!"
        exit 1
        ;;
esac

echo ""
echo "🎉 Build process completed successfully!"
echo "📦 Distribution packages are ready in the 'build' directory."
