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
    <!-- Ensure Tesseract finds tessdata within the app bundle -->
    <key>LSEnvironment</key>
    <dict>
        <key>TESSDATA_PREFIX</key>
        <string>@@EXECUTABLE_PATH@@/../Resources/assets/tesseract</string>
    </dict>
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

# Bundle Tesseract/Leptonica dylibs to make the app self-contained
echo "🧩 Bundling Tesseract/Leptonica frameworks..."

DYLIBS_DIR="$APP_BUNDLE/Contents/Frameworks"
mkdir -p "$DYLIBS_DIR"

# Locate dylibs from common Homebrew paths (Apple Silicon and Intel),
# with fallbacks to versioned names and brew --prefix
TESSERACT_LIB=${TESSERACT_LIB:-}
LEPTONICA_LIB=${LEPTONICA_LIB:-}

if [ -z "$TESSERACT_LIB" ]; then
  for PATH_CAND in \
    "/opt/homebrew/opt/tesseract/lib/libtesseract.dylib" \
    "/usr/local/opt/tesseract/lib/libtesseract.dylib"; do
    if [ -f "$PATH_CAND" ]; then TESSERACT_LIB="$PATH_CAND"; break; fi
  done
fi

if [ -z "$TESSERACT_LIB" ]; then
  BREW_TESS_PREFIX=$(brew --prefix tesseract 2>/dev/null || true)
  if [ -d "$BREW_TESS_PREFIX/lib" ]; then
    # pick first matching versioned dylib
    TESSERACT_LIB=$(ls "$BREW_TESS_PREFIX"/lib/libtesseract*.dylib 2>/dev/null | head -n1 || true)
  fi
fi

if [ -z "$LEPTONICA_LIB" ]; then
  for PATH_CAND in \
    "/opt/homebrew/opt/leptonica/lib/liblept.dylib" \
    "/usr/local/opt/leptonica/lib/liblept.dylib"; do
    if [ -f "$PATH_CAND" ]; then LEPTONICA_LIB="$PATH_CAND"; break; fi
  done
fi

if [ -z "$LEPTONICA_LIB" ]; then
  BREW_LEPT_PREFIX=$(brew --prefix leptonica 2>/dev/null || true)
  if [ -d "$BREW_LEPT_PREFIX/lib" ]; then
    LEPTONICA_LIB=$(ls "$BREW_LEPT_PREFIX"/lib/liblept*.dylib 2>/dev/null | head -n1 || true)
  fi
fi

# As a last resort for Leptonica, discover via libtesseract's dependencies
if [ -z "$LEPTONICA_LIB" ] && [ -n "$TESSERACT_LIB" ]; then
  LEPTONICA_LIB=$(otool -L "$TESSERACT_LIB" 2>/dev/null | awk '{print $1}' | grep -E 'liblept.*\\.dylib' | head -n1 || true)
fi

if [ -z "$TESSERACT_LIB" ] || [ -z "$LEPTONICA_LIB" ]; then
  echo "❌ Could not locate libtesseract.dylib or liblept.dylib in Homebrew locations."
  echo "   Please install via: brew install tesseract leptonica"
  echo "   Or set TESSERACT_LIB and LEPTONICA_LIB env vars to the dylib paths, then re-run."
  if [ -n "$TESSERACT_LIB" ]; then echo "   Found Tesseract at: $TESSERACT_LIB"; fi
  if [ -n "$LEPTONICA_LIB" ]; then echo "   Found Leptonica at: $LEPTONICA_LIB"; fi
  exit 1
fi

echo "   Tesseract:  $TESSERACT_LIB"
echo "   Leptonica:  $LEPTONICA_LIB"

TESS_BASE=$(basename "$TESSERACT_LIB")
LEPT_BASE=$(basename "$LEPTONICA_LIB")

cp "$TESSERACT_LIB" "$DYLIBS_DIR/$TESS_BASE"
cp "$LEPTONICA_LIB" "$DYLIBS_DIR/$LEPT_BASE"

# Copy any dependent dylibs of libtesseract/liblept that live under Homebrew (to reduce external deps)
copy_deps() {
  local lib_path="$1"
  otool -L "$lib_path" | awk '{print $1}' | grep -E "/(opt|Cellar)/" | while read -r dep; do
    base=$(basename "$dep")
    if [ ! -f "$DYLIBS_DIR/$base" ]; then
      if [ -f "$dep" ]; then
        echo "   📦 Copying dependency: $dep"
        cp "$dep" "$DYLIBS_DIR/$base" || true
      fi
    fi
  done
}

copy_deps "$DYLIBS_DIR/$TESS_BASE"
copy_deps "$DYLIBS_DIR/$LEPT_BASE"

# Adjust rpaths and install names so the app uses bundled frameworks
echo "🛠️  Rewriting library references..."

# Add rpath to app binary
install_name_tool -add_rpath "@executable_path/../Frameworks" "$APP_BUNDLE/Contents/MacOS/$BINARY_NAME" || true

# Point app binary to bundled libs (use discovered basenames)
for REF in $(otool -L "$APP_BUNDLE/Contents/MacOS/$BINARY_NAME" | awk '{print $1}' | grep -E "libtesseract.*dylib" || true); do
  echo "   🔗 Rewriting $REF -> @rpath/$TESS_BASE"
  install_name_tool -change "$REF" "@rpath/$TESS_BASE" "$APP_BUNDLE/Contents/MacOS/$BINARY_NAME" || true
done
for REF in $(otool -L "$APP_BUNDLE/Contents/MacOS/$BINARY_NAME" | awk '{print $1}' | grep -E "liblept.*dylib" || true); do
  echo "   🔗 Rewriting $REF -> @rpath/$LEPT_BASE"
  install_name_tool -change "$REF" "@rpath/$LEPT_BASE" "$APP_BUNDLE/Contents/MacOS/$BINARY_NAME" || true
done

# Ensure the bundled libs reference each other via @loader_path
for LIB in "$DYLIBS_DIR"/*.dylib; do
  # Change references to Homebrew libs inside each dylib to local copies
  otool -L "$LIB" | awk '{print $1}' | grep -E "(libtesseract|liblept).*dylib" | while read -r REF; do
    base=$(basename "$REF")
    echo "   🧶 Rewriting $LIB needs $REF -> @loader_path/$base"
    install_name_tool -change "$REF" "@loader_path/$base" "$LIB" || true
  done
  # Also set install_name of each dylib to @rpath form
  base=$(basename "$LIB")
  install_name_tool -id "@rpath/$base" "$LIB" || true
done

# Ad-hoc sign to satisfy Gatekeeper's basic checks (not notarized)
echo "✍️  Ad-hoc signing app bundle..."
codesign --force --deep -s - "$APP_BUNDLE" || true

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
