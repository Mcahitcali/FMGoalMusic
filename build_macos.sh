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
DATE_TAG="$(date +%Y%m%d)"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Build function per-arch
build_for_arch() {
  local ARCH="$1"                # arm64 or x86_64
  local HB_PREFIX="$2"           # /opt/homebrew (arm64) or /usr/local (intel)
  local TARGET_TRIPLE="$3"       # empty for native or x86_64-apple-darwin

  echo "🔨 Building release binary for $ARCH..."
  if [ -n "$TARGET_TRIPLE" ]; then
    if ! rustup target list --installed | grep -q "$TARGET_TRIPLE"; then
      echo "ℹ️  Rust target $TARGET_TRIPLE not installed. Install with: rustup target add $TARGET_TRIPLE"
      return 2
    fi
    cargo build --release --target "$TARGET_TRIPLE" --bin "$BINARY_NAME"
    BIN_PATH="target/$TARGET_TRIPLE/release/$BINARY_NAME"
  else
    cargo build --release --bin "$BINARY_NAME"
    BIN_PATH="target/release/$BINARY_NAME"
  fi

  # Prepare per-arch build dir
  local ARCH_BUILD_DIR="$BUILD_DIR-$ARCH"
  local ARCH_APP_BUNDLE="$ARCH_BUILD_DIR/$APP_NAME.app"
  rm -rf "$ARCH_BUILD_DIR"
  mkdir -p "$ARCH_APP_BUNDLE/Contents/MacOS" "$ARCH_APP_BUNDLE/Contents/Resources" "$ARCH_APP_BUNDLE/Contents/Frameworks"

  # Info.plist
  cat > "$ARCH_APP_BUNDLE/Contents/Info.plist" << EOF
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
    <key>LSEnvironment</key>
    <dict>
        <key>TESSDATA_PREFIX</key>
        <string>@@EXECUTABLE_PATH@@/../Resources/assets/tesseract</string>
    </dict>
</dict>
</plist>
EOF

  echo "📋 Copying binary..."
  cp "$BIN_PATH" "$ARCH_APP_BUNDLE/Contents/MacOS/"

  if [ -f "assets/icon.icns" ]; then
    cp "assets/icon.icns" "$ARCH_APP_BUNDLE/Contents/Resources/AppIcon.icns"
  fi

  if [ -f "goal_crowd_cheer.wav" ]; then
    cp "goal_crowd_cheer.wav" "$ARCH_APP_BUNDLE/Contents/Resources/"
  fi

  if [ -d "assets" ]; then
    cp -r assets "$ARCH_APP_BUNDLE/Contents/Resources/"
  fi

  # Bundle dylibs
  echo "🧩 Bundling Tesseract/Leptonica frameworks for $ARCH..."
  DYLIBS_DIR="$ARCH_APP_BUNDLE/Contents/Frameworks"
  mkdir -p "$DYLIBS_DIR"

  TESSERACT_LIB=${TESSERACT_LIB:-}
  LEPTONICA_LIB=${LEPTONICA_LIB:-}

  if [ -z "$TESSERACT_LIB" ]; then
    for PATH_CAND in \
      "$HB_PREFIX/opt/tesseract/lib/libtesseract.dylib" \
      "$HB_PREFIX/opt/tesseract/lib/libtesseract"*.dylib; do
      [ -f "$PATH_CAND" ] && TESSERACT_LIB="$PATH_CAND" && break
    done
  fi
  if [ -z "$TESSERACT_LIB" ]; then
    BREW_TESS_PREFIX=$(brew --prefix tesseract 2>/dev/null || true)
    [ -d "$BREW_TESS_PREFIX/lib" ] && TESSERACT_LIB=$(ls "$BREW_TESS_PREFIX"/lib/libtesseract*.dylib 2>/dev/null | head -n1 || true)
  fi

  if [ -z "$LEPTONICA_LIB" ]; then
    for PATH_CAND in \
      "$HB_PREFIX/opt/leptonica/lib/liblept.dylib" \
      "$HB_PREFIX/opt/leptonica/lib/liblept"*.dylib \
      "$HB_PREFIX/opt/leptonica/lib/libleptonica"*.dylib; do
      [ -f "$PATH_CAND" ] && LEPTONICA_LIB="$PATH_CAND" && break
    done
  fi
  if [ -z "$LEPTONICA_LIB" ] && [ -n "$TESSERACT_LIB" ]; then
    LEPTONICA_LIB=$(otool -L "$TESSERACT_LIB" 2>/dev/null | awk '{print $1}' | grep -E 'liblept|libleptonica' | head -n1 || true)
  fi

  if [ -z "$TESSERACT_LIB" ] || [ -z "$LEPTONICA_LIB" ]; then
    echo "⚠️  Skipping $ARCH bundle: could not locate appropriate libtesseract/liblept under $HB_PREFIX."
    return 3
  fi

  echo "   Tesseract:  $TESSERACT_LIB"
  echo "   Leptonica:  $LEPTONICA_LIB"

  TESS_BASE=$(basename "$TESSERACT_LIB")
  LEPT_BASE=$(basename "$LEPTONICA_LIB")
  cp "$TESSERACT_LIB" "$DYLIBS_DIR/$TESS_BASE"
  cp "$LEPTONICA_LIB" "$DYLIBS_DIR/$LEPT_BASE"

  copy_deps() {
    local lib_path="$1"
    otool -L "$lib_path" | awk '{print $1}' | grep -E "/(opt|Cellar)/" | while read -r dep; do
      base=$(basename "$dep")
      [ -f "$DYLIBS_DIR/$base" ] || { echo "   📦 Copying dependency: $dep"; cp "$dep" "$DYLIBS_DIR/$base" || true; }
    done
  }
  copy_deps "$DYLIBS_DIR/$TESS_BASE"
  copy_deps "$DYLIBS_DIR/$LEPT_BASE"

  echo "🛠️  Rewriting library references..."
  install_name_tool -add_rpath "@executable_path/../Frameworks" "$ARCH_APP_BUNDLE/Contents/MacOS/$BINARY_NAME" || true
  for REF in $(otool -L "$ARCH_APP_BUNDLE/Contents/MacOS/$BINARY_NAME" | awk '{print $1}' | grep -E "libtesseract.*dylib" || true); do
    install_name_tool -change "$REF" "@rpath/$TESS_BASE" "$ARCH_APP_BUNDLE/Contents/MacOS/$BINARY_NAME" || true
  done
  for REF in $(otool -L "$ARCH_APP_BUNDLE/Contents/MacOS/$BINARY_NAME" | awk '{print $1}' | grep -E "liblept|libleptonica.*dylib" || true); do
    install_name_tool -change "$REF" "@rpath/$LEPT_BASE" "$ARCH_APP_BUNDLE/Contents/MacOS/$BINARY_NAME" || true
  done
  for LIB in "$DYLIBS_DIR"/*.dylib; do
    otool -L "$LIB" | awk '{print $1}' | grep -E "(libtesseract|liblept|libleptonica).*dylib" | while read -r REF; do
      base=$(basename "$REF")
      install_name_tool -change "$REF" "@loader_path/$base" "$LIB" || true
    done
    base=$(basename "$LIB")
    install_name_tool -id "@rpath/$base" "$LIB" || true
  done

  echo "✍️  Ad-hoc signing app bundle..."
  codesign --force --deep -s - "$ARCH_APP_BUNDLE" || true

  echo "🔐 Setting permissions..."
  chmod +x "$ARCH_APP_BUNDLE/Contents/MacOS/$BINARY_NAME"

  echo "💿 Creating DMG..."
  DMG_DIR="$ARCH_BUILD_DIR/dmg_temp"
  mkdir -p "$DMG_DIR"
  cp -R "$ARCH_APP_BUNDLE" "$DMG_DIR/"
  ln -s /Applications "$DMG_DIR/Applications"
  DMG_NAME="$ARCH_BUILD_DIR/$APP_NAME-$ARCH-$DATE_TAG.dmg"
  hdiutil create -volname "$APP_NAME" -srcfolder "$DMG_DIR" -ov -format UDZO "$DMG_NAME"
  rm -rf "$DMG_DIR"

  echo "✅ Done for $ARCH"
  echo "📍 App bundle: $ARCH_APP_BUNDLE"
  echo "💿 DMG file: $DMG_NAME"
}

# Run builds
echo "📦 Creating per-arch builds..."
ARM_OK=0; X64_OK=0

# Apple Silicon build (arm64)
build_for_arch "arm64" "/opt/homebrew" "" && ARM_OK=1 || true

# Intel build (x86_64) — requires Intel Homebrew (/usr/local) and rust target
build_for_arch "x86_64" "/usr/local" "x86_64-apple-darwin" && X64_OK=1 || true

echo ""
echo "✅ Build summary:" 
if [ "$ARM_OK" = "1" ]; then echo " - arm64 DMG created under $BUILD_DIR-arm64"; else echo " - arm64 build skipped/failed"; fi
if [ "$X64_OK" = "1" ]; then echo " - x86_64 DMG created under $BUILD_DIR-x86_64"; else echo " - x86_64 build skipped/failed (install Intel Homebrew under /usr/local and run: rustup target add x86_64-apple-darwin)"; fi

echo ""
echo "📋 Installation Instructions:"
echo "1. Open the DMG file"
echo "2. Drag '$APP_NAME.app' to the Applications folder shortcut"
echo "3. Find the app in your Applications folder"
