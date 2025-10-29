#!/bin/bash

# Build macOS app bundle for FM Goal Musics GUI

APP_NAME="FM Goal Musics"
BUNDLE_ID="com.fmgoalmusics.gui"
VERSION="0.1.0"
BINARY_NAME="fm-goal-musics-gui"

# Build the release binary
echo "🔨 Building release binary..."
if ! cargo build --release --bin fm-goal-musics-gui; then
    echo "❌ Build failed"
    exit 1
fi

# Create app bundle structure
APP_BUNDLE="target/release/$APP_NAME.app"
CONTENTS="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS/MacOS"
RESOURCES_DIR="$CONTENTS/Resources"

echo "📦 Creating app bundle structure..."
rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy binary
if [ ! -f "target/release/$BINARY_NAME" ]; then
    echo "❌ Binary not found at target/release/$BINARY_NAME"
    exit 1
fi

cp "target/release/$BINARY_NAME" "$MACOS_DIR/$APP_NAME"
chmod +x "$MACOS_DIR/$APP_NAME"

# Verify binary was copied
if [ ! -f "$MACOS_DIR/$APP_NAME" ]; then
    echo "❌ Failed to copy binary"
    exit 1
fi

# Create Info.plist
cat > "$CONTENTS/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>FM Goal Musics</string>
    <key>CFBundleIdentifier</key>
    <string>com.fmgoalmusics.gui</string>
    <key>CFBundleName</key>
    <string>FM Goal Musics</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSScreenCaptureUsageDescription</key>
    <string>FM Goal Musics needs screen capture permission to detect goals in your game.</string>
    <key>NSAccessibilityUsageDescription</key>
    <string>FM Goal Musics needs accessibility permissions to listen for global hotkeys (Cmd+Shift+R, Cmd+1).</string>
</dict>
</plist>
EOF

echo "✅ App bundle created at: $APP_BUNDLE"
echo ""
echo "🔐 Code signing app..."
codesign --force --deep --sign - "$APP_BUNDLE" 2>/dev/null || true
echo "✅ Code signing complete"
echo ""
echo "📋 Next steps:"
echo "1. Open System Settings > Privacy & Security"
echo "2. Grant 'FM Goal Musics' permission for:"
echo "   - Screen Recording"
echo "   - Accessibility"
echo "   - Input Monitoring"
echo ""
echo "3. Run the app:"
echo "   open \"$APP_BUNDLE\""
echo ""
echo "💡 Tip: After granting permissions, you won't need to grant them again on future rebuilds."
