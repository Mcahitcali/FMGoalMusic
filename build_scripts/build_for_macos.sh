#!/usr/bin/env bash
set -euo pipefail

# Simple macOS .app + .dmg builder for FM Goal Musics
# Usage:
#   ./build_scripts/build_for_macos.sh            # default (arm64)
#   ARCH=x86_64 ./build_scripts/build_for_macos.sh  # Intel build (run from Rosetta terminal)

APP_NAME="FM Goal Musics"
BUNDLE_ID="com.fmgoalmusic.app"  # TODO: adjust if you have a specific bundle id
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Architecture selection: arm64 (Apple Silicon) vs x86_64 (Intel)
ARCH="${ARCH:-arm64}"
case "$ARCH" in
  arm64)
    BREW_PREFIX="/opt/homebrew"
    DMG_SUFFIX="-arm64"
    ;;
  x86_64)
    BREW_PREFIX="/usr/local"
    DMG_SUFFIX="-Intel"
    ;;
  *)
    echo "Unsupported ARCH: $ARCH (expected arm64 or x86_64)" >&2
    exit 1
    ;;
esac

TARGET_DIR="$ROOT_DIR/target/release"
APP_DIR="$ROOT_DIR/build/macos/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
FRAMEWORKS_DIR="$CONTENTS_DIR/Frameworks"
DMG_PATH="$ROOT_DIR/build/macos/FMGoalMusics${DMG_SUFFIX}.dmg"

echo "=== Building fm-goal-musics-gui (release) for ARCH=$ARCH ==="

# For Intel build, make sure pkg-config prefers Intel Homebrew under /usr/local
if [ "$ARCH" = "x86_64" ]; then
  export PKG_CONFIG_PATH="/usr/local/lib/pkgconfig:/usr/local/opt/leptonica/lib/pkgconfig:/usr/local/opt/tesseract/lib/pkgconfig:/usr/local/opt/libarchive/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
fi

(cd "$ROOT_DIR" && cargo build --release --bin fm-goal-musics-gui)

# Clean previous app/dmg
rm -rf "$APP_DIR" "$DMG_PATH"
mkdir -p "$MACOS_DIR" "$RESOURCES_DIR" "$FRAMEWORKS_DIR"

echo "=== Staging .app bundle ==="

# 1. Copy binary
cp "$TARGET_DIR/fm-goal-musics-gui" "$MACOS_DIR/"
chmod +x "$MACOS_DIR/fm-goal-musics-gui"

# 2. Copy resources
# Copy tessdata directory (for bundled Tesseract)
if [ -d "$ROOT_DIR/tessdata" ]; then
  echo "Copying tessdata -> $RESOURCES_DIR/tessdata"
  cp -R "$ROOT_DIR/tessdata" "$RESOURCES_DIR/tessdata"
else
  echo "WARNING: tessdata directory not found at $ROOT_DIR/tessdata"
fi

# Copy assets (icons, etc.)
if [ -d "$ROOT_DIR/assets" ]; then
  echo "Copying assets -> $RESOURCES_DIR/assets"
  cp -R "$ROOT_DIR/assets" "$RESOURCES_DIR/assets"
fi

echo "=== Bundling Homebrew dynamic libraries ==="

# Helper to copy a dylib into Frameworks and adjust its install name
bundle_dylib() {
  local src="$1"      # full path to source dylib
  local name
  name="$(basename "$src")"

  if [ ! -f "$src" ]; then
    echo "WARNING: dylib not found: $src"
    return
  fi

  echo "Bundling $name from $src"
  cp "$src" "$FRAMEWORKS_DIR/$name"

  # Set the dylib's own install name to @rpath
  install_name_tool -id "@rpath/$name" "$FRAMEWORKS_DIR/$name"

  # Point the main executable to this bundled dylib
  install_name_tool -change "$src" "@executable_path/../Frameworks/$name" "$MACOS_DIR/fm-goal-musics-gui"
}

# Helper to scan a single dylib for Homebrew-linked deps and rewrite them to bundled copies
fix_homebrew_deps_for_dylib() {
  local target="$1"   # path to the dylib inside Frameworks

  # Use otool -L to list dependencies and pick those under $BREW_PREFIX
  while IFS= read -r line; do
    # Example line: "\t/opt/homebrew/opt/zstd/lib/libzstd.1.dylib (compatibility ..."
    case "$line" in
      *"$BREW_PREFIX"*.dylib*)
        local dep
        dep="$(echo "$line" | awk '{print $1}')"
        # Skip if we failed to parse
        if [ -z "$dep" ]; then
          continue
        fi

        local dep_name
        dep_name="$(basename "$dep")"

        # Bundle the dependency itself
        bundle_dylib "$dep"

        # Rewrite this dependency inside the target dylib to point to the bundled copy
        install_name_tool -change "$dep" "@executable_path/../Frameworks/$dep_name" "$target" || true
        ;;
    esac
  done < <(otool -L "$target" 2>/dev/null)
}

# Known external libs from otool -L
bundle_dylib "$BREW_PREFIX/opt/libarchive/lib/libarchive.13.dylib"
bundle_dylib "$BREW_PREFIX/opt/tesseract/lib/libtesseract.5.dylib"
bundle_dylib "$BREW_PREFIX/opt/leptonica/lib/libleptonica.6.dylib"

# Bundle leptonica image codec dependencies and adjust their paths inside libleptonica/libtesseract
bundle_dylib "$BREW_PREFIX/opt/libpng/lib/libpng16.16.dylib"
bundle_dylib "$BREW_PREFIX/opt/jpeg-turbo/lib/libjpeg.8.dylib"
bundle_dylib "$BREW_PREFIX/opt/giflib/lib/libgif.dylib"
bundle_dylib "$BREW_PREFIX/opt/libtiff/lib/libtiff.6.dylib"
bundle_dylib "$BREW_PREFIX/opt/webp/lib/libwebp.7.dylib"
bundle_dylib "$BREW_PREFIX/opt/webp/lib/libwebpmux.3.dylib"
bundle_dylib "$BREW_PREFIX/opt/webp/lib/libsharpyuv.0.dylib"
bundle_dylib "$BREW_PREFIX/opt/openjpeg/lib/libopenjp2.7.dylib"

# libarchive dependencies
bundle_dylib "$BREW_PREFIX/opt/xz/lib/liblzma.5.dylib"
bundle_dylib "$BREW_PREFIX/opt/zstd/lib/libzstd.1.dylib"
bundle_dylib "$BREW_PREFIX/opt/lz4/lib/liblz4.1.dylib"
bundle_dylib "$BREW_PREFIX/opt/libb2/lib/libb2.1.dylib"

echo "=== Fixing nested dylib paths inside frameworks ==="

# Update libleptonica to point to bundled codecs instead of Homebrew paths
if [ -f "$FRAMEWORKS_DIR/libleptonica.6.dylib" ]; then
  install_name_tool -change \
    "$BREW_PREFIX/opt/libpng/lib/libpng16.16.dylib" \
    "@executable_path/../Frameworks/libpng16.16.dylib" \
    "$FRAMEWORKS_DIR/libleptonica.6.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/jpeg-turbo/lib/libjpeg.8.dylib" \
    "@executable_path/../Frameworks/libjpeg.8.dylib" \
    "$FRAMEWORKS_DIR/libleptonica.6.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/giflib/lib/libgif.dylib" \
    "@executable_path/../Frameworks/libgif.dylib" \
    "$FRAMEWORKS_DIR/libleptonica.6.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/libtiff/lib/libtiff.6.dylib" \
    "@executable_path/../Frameworks/libtiff.6.dylib" \
    "$FRAMEWORKS_DIR/libleptonica.6.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/webp/lib/libwebp.7.dylib" \
    "@executable_path/../Frameworks/libwebp.7.dylib" \
    "$FRAMEWORKS_DIR/libleptonica.6.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/webp/lib/libwebpmux.3.dylib" \
    "@executable_path/../Frameworks/libwebpmux.3.dylib" \
    "$FRAMEWORKS_DIR/libleptonica.6.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/openjpeg/lib/libopenjp2.7.dylib" \
    "@executable_path/../Frameworks/libopenjp2.7.dylib" \
    "$FRAMEWORKS_DIR/libleptonica.6.dylib" || true
fi

# Update libtiff to point to bundled libzstd, liblzma and libjpeg instead of Homebrew paths
if [ -f "$FRAMEWORKS_DIR/libtiff.6.dylib" ]; then
  install_name_tool -change \
    "$BREW_PREFIX/opt/zstd/lib/libzstd.1.dylib" \
    "@executable_path/../Frameworks/libzstd.1.dylib" \
    "$FRAMEWORKS_DIR/libtiff.6.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/xz/lib/liblzma.5.dylib" \
    "@executable_path/../Frameworks/liblzma.5.dylib" \
    "$FRAMEWORKS_DIR/libtiff.6.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/jpeg-turbo/lib/libjpeg.8.dylib" \
    "@executable_path/../Frameworks/libjpeg.8.dylib" \
    "$FRAMEWORKS_DIR/libtiff.6.dylib" || true
fi

# Update libwebpmux and libwebp to point to bundled libsharpyuv instead of @rpath
if [ -f "$FRAMEWORKS_DIR/libwebpmux.3.dylib" ]; then
  install_name_tool -change \
    "@rpath/libsharpyuv.0.dylib" \
    "@executable_path/../Frameworks/libsharpyuv.0.dylib" \
    "$FRAMEWORKS_DIR/libwebpmux.3.dylib" || true
fi

if [ -f "$FRAMEWORKS_DIR/libwebp.7.dylib" ]; then
  install_name_tool -change \
    "@rpath/libsharpyuv.0.dylib" \
    "@executable_path/../Frameworks/libsharpyuv.0.dylib" \
    "$FRAMEWORKS_DIR/libwebp.7.dylib" || true
fi

# Update libtesseract to point to bundled leptonica/archive instead of Homebrew paths
if [ -f "$FRAMEWORKS_DIR/libtesseract.5.dylib" ]; then
  install_name_tool -change \
    "$BREW_PREFIX/opt/leptonica/lib/libleptonica.6.dylib" \
    "@executable_path/../Frameworks/libleptonica.6.dylib" \
    "$FRAMEWORKS_DIR/libtesseract.5.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/libarchive/lib/libarchive.13.dylib" \
    "@executable_path/../Frameworks/libarchive.13.dylib" \
    "$FRAMEWORKS_DIR/libtesseract.5.dylib" || true
fi

# Update libarchive to point to bundled compression libs instead of Homebrew paths
if [ -f "$FRAMEWORKS_DIR/libarchive.13.dylib" ]; then
  install_name_tool -change \
    "$BREW_PREFIX/opt/xz/lib/liblzma.5.dylib" \
    "@executable_path/../Frameworks/liblzma.5.dylib" \
    "$FRAMEWORKS_DIR/libarchive.13.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/zstd/lib/libzstd.1.dylib" \
    "@executable_path/../Frameworks/libzstd.1.dylib" \
    "$FRAMEWORKS_DIR/libarchive.13.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/lz4/lib/liblz4.1.dylib" \
    "@executable_path/../Frameworks/liblz4.1.dylib" \
    "$FRAMEWORKS_DIR/libarchive.13.dylib" || true

  install_name_tool -change \
    "$BREW_PREFIX/opt/libb2/lib/libb2.1.dylib" \
    "@executable_path/../Frameworks/libb2.1.dylib" \
    "$FRAMEWORKS_DIR/libarchive.13.dylib" || true
fi

# As a safety net, run a generic Homebrew dep fixer over all bundled frameworks.
# This will catch any remaining $BREW_PREFIX/*.dylib references and rewrite them
# to use the copies we bundle into Contents/Frameworks.
for dylib in "$FRAMEWORKS_DIR"/*.dylib; do
  if [ -f "$dylib" ]; then
    fix_homebrew_deps_for_dylib "$dylib"
  fi
done

# 3. Minimal Info.plist
cat > "$CONTENTS_DIR/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>fm-goal-musics-gui</string>
  <key>CFBundleIdentifier</key>
  <string>$BUNDLE_ID</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>$APP_NAME</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleIconFile</key>
  <string>app</string>
  <key>CFBundleShortVersionString</key>
  <string>1.0.0</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>LSMinimumSystemVersion</key>
  <string>10.15</string>
</dict>
</plist>
EOF

# 4. Optional: use app.icns if you already have one under assets/icons
if [ -f "$ROOT_DIR/assets/app.icns" ]; then
  echo "Copying app.icns -> $RESOURCES_DIR/app.icns"
  cp "$ROOT_DIR/assets/app.icns" "$RESOURCES_DIR/app.icns"
elif [ -f "$ROOT_DIR/assets/icon.icns" ]; then
  echo "Copying icon.icns -> $RESOURCES_DIR/app.icns"
  cp "$ROOT_DIR/assets/icon.icns" "$RESOURCES_DIR/app.icns"
fi

echo "=== Codesign with Developer ID Application certificate ==="
CERT="Developer ID Application: Mehmet Mucahit Cali (799WAZ6G47)"

# 1) Sign all bundled frameworks (Homebrew dylibs) so they share the same Team ID
for dylib in "$FRAMEWORKS_DIR"/*.dylib; do
  if [ -f "$dylib" ]; then
    codesign --force --options runtime --sign "$CERT" "$dylib"
  fi
done

# 2) Sign the .app bundle (recursively) with the same identity
codesign --force --deep --options runtime --sign "$CERT" "$APP_DIR"

echo "=== Creating DMG (with Applications alias) ==="
mkdir -p "$ROOT_DIR/build/macos"

DMG_STAGING_DIR="$ROOT_DIR/build/macos/dmg_src"
TMP_DMG="$ROOT_DIR/build/macos/FMGoalMusics-tmp.dmg"

rm -rf "$DMG_STAGING_DIR" "$TMP_DMG" "$DMG_PATH"
mkdir -p "$DMG_STAGING_DIR"

# Copy app into staging
cp -R "$APP_DIR" "$DMG_STAGING_DIR/"

# Create Applications symlink inside DMG root
ln -s /Applications "$DMG_STAGING_DIR/Applications"

# Set a custom DMG icon if we have one
if [ -f "$ROOT_DIR/assets/icon.icns" ]; then
  cp "$ROOT_DIR/assets/icon.icns" "$DMG_STAGING_DIR/.VolumeIcon.icns"
  # Mark the directory so Finder uses the custom icon (SetFile is part of Xcode command line tools)
  if command -v SetFile >/dev/null 2>&1; then
    SetFile -a C "$DMG_STAGING_DIR"
  fi
fi

# Create final compressed DMG directly from the staging folder
echo "Creating final compressed DMG from staging folder..."
hdiutil create -volname "$APP_NAME" \
  -srcfolder "$DMG_STAGING_DIR" \
  -ov -format UDZO "$DMG_PATH"

echo "=== Notarization (optional) ==="
if [ -n "${FMGOAL_NOTARY_APPLE_ID:-}" ] && [ -n "${FMGOAL_NOTARY_PASSWORD:-}" ] && [ -n "${FMGOAL_NOTARY_TEAM_ID:-}" ]; then
  echo "Submitting DMG to Apple notarization service..."
  xcrun notarytool submit "$DMG_PATH" \
    --apple-id "$FMGOAL_NOTARY_APPLE_ID" \
    --team-id "$FMGOAL_NOTARY_TEAM_ID" \
    --password "$FMGOAL_NOTARY_PASSWORD" \
    --wait

  echo "Stapling notarization ticket to .app and .dmg..."
  xcrun stapler staple "$APP_DIR" || true
  xcrun stapler staple "$DMG_PATH" || true
else
  echo "Skipping notarization (set FMGOAL_NOTARY_APPLE_ID, FMGOAL_NOTARY_PASSWORD, FMGOAL_NOTARY_TEAM_ID to enable)"
fi

echo "=== Done ==="
echo "App bundle: $APP_DIR"
echo "DMG file : $DMG_PATH"
