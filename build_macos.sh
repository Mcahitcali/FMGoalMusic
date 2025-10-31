#!/bin/bash

# FM Goal Musics - macOS Build Script
# Creates a distributable .app bundle for macOS

set -e

echo "🍎 Building FM Goal Musics for macOS..."

# Configuration
APP_NAME="FM Goal Musics"
BUNDLE_ID="com.fmgoalmusics.app"
BINARY_NAME="fm-goal-musics-gui"
SOURCE_DIR="src"
TARGET_DIR="target/release"
BUILD_DIR="build/macos"
APP_BUNDLE="$BUILD_DIR/$APP_NAME.app"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Build the release binary
echo "🔨 Building release binary..."
cargo build --release --bin "$BINARY_NAME"

# Create app bundle structure
echo "📦 Creating app bundle structure..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"
mkdir -p "$APP_BUNDLE/Contents/Frameworks"

# Create Info.plist
echo "📄 Creating Info.plist..."
cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleExecutable</key>
    <string>$BINARY_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
</dict>
</plist>
EOF

# Copy binary to app bundle
echo "📋 Copying binary..."
cp "$TARGET_DIR/$BINARY_NAME" "$APP_BUNDLE/Contents/MacOS/"

# Create a simple icon (you can replace this with a proper .icns file)
echo "🎨 Creating placeholder icon..."
# Create a simple 512x512 PNG icon (you should replace this with a proper icon)
if [ ! -f "assets/icon.icns" ]; then
    echo "⚠️  No icon found at assets/icon.icns - using placeholder"
    # Create minimal icon or skip
    mkdir -p assets
    # Note: You should add a proper .icns file here
fi

# Copy icon if exists
if [ -f "assets/icon.icns" ]; then
    cp "assets/icon.icns" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
fi

# Copy default ambiance sound
if [ -f "goal_crowd_cheer.wav" ]; then
    echo "🎵 Copying default ambiance sound..."
    cp "goal_crowd_cheer.wav" "$APP_BUNDLE/Contents/Resources/"
fi

# Copy necessary resources
echo "📚 Copying resources..."
if [ -d "assets" ]; then
    cp -r assets "$APP_BUNDLE/Contents/Resources/"
fi

# Set executable permissions
echo "🔐 Setting permissions..."
chmod +x "$APP_BUNDLE/Contents/MacOS/$BINARY_NAME"

# Create DMG with Applications shortcut for easy installation
echo "💿 Creating DMG with installation interface..."
DMG_DIR="$BUILD_DIR/dmg_temp"
mkdir -p "$DMG_DIR"

# Copy app to temporary DMG directory
cp -R "$APP_BUNDLE" "$DMG_DIR/"

# Create symbolic link to Applications folder
ln -s /Applications "$DMG_DIR/Applications"

# Create DMG
DMG_NAME="$BUILD_DIR/$APP_NAME-$(date +%Y%m%d).dmg"
hdiutil create -volname "$APP_NAME" -srcfolder "$DMG_DIR" -ov -format UDZO "$DMG_NAME"

# Clean up temporary directory
rm -rf "$DMG_DIR"

echo "✅ Enhanced macOS build completed successfully!"
echo "📍 App bundle: $APP_BUNDLE"
echo "💿 DMG file: $DMG_NAME"
echo ""
echo "📋 Installation Instructions:"
echo "1. Open the DMG file"
echo "2. Drag '$APP_NAME.app' to the Applications folder shortcut"
echo "3. Find the app in your Applications folder"
echo ""
echo "To test the app:"
echo "open \"$APP_BUNDLE\""
echo ""
echo "To install the DMG:"
echo "open \"$DMG_NAME\""
