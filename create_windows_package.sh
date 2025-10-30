#!/bin/bash

# Create Windows-ready source package
# This creates a ZIP with all source files and build scripts for Windows

echo "📦 Creating Windows-ready source package..."

# Clean previous package
rm -f "FM-Goal-Musics-Windows-Source-$(date +%Y%m%d).zip"

# Create temporary directory
TEMP_DIR="FM-Goal-Musics-Windows-Source"
rm -rf "$TEMP_DIR"
mkdir -p "$TEMP_DIR"

# Copy source files
echo "📋 Copying source files..."
cp -r src/ "$TEMP_DIR/"
cp Cargo.toml "$TEMP_DIR/"
cp Cargo.lock "$TEMP_DIR/" 2>/dev/null || echo "⚠️  No Cargo.lock found"

# Copy build scripts
echo "🔨 Copying build scripts..."
cp build_windows.bat "$TEMP_DIR/"
cp BUILD.md "$TEMP_DIR/"

# Copy assets
echo "🎨 Copying assets..."
cp -r assets/ "$TEMP_DIR/"

# Copy README
echo "📄 Creating Windows README..."
cat > "$TEMP_DIR/README-Windows.md" << 'EOF'
# FM Goal Musics - Windows Installation

## Quick Start (No Rust knowledge needed!)

### Prerequisites:
- Windows 10 or 11
- Internet connection (one-time setup)

### Installation Steps:

1. **Install Rust** (one-time setup):
   - Download and run: https://rustup.rs/
   - Restart command prompt after installation

2. **Build the application**:
   ```batch
   build_windows.bat
   ```

3. **Run the app**:
   - Extract the generated ZIP file
   - Double-click `fm-goal-musics-gui.exe`

### What You Get:
- Self-contained Windows application
- No external dependencies needed
- Includes OCR functionality
- Portable - no installation required

### Need Help?
- Check BUILD.md for detailed instructions
- All features work out of the box

Enjoy your goal celebration music! 🎵⚽
EOF

# Create ZIP package
echo "🗜️  Creating ZIP package..."
zip -r "FM-Goal-Musics-Windows-Source-$(date +%Y%m%d).zip" "$TEMP_DIR"

# Clean up
rm -rf "$TEMP_DIR"

echo "✅ Windows source package created!"
echo "📍 File: FM-Goal-Musics-Windows-Source-$(date +%Y%m%d).zip"
echo ""
echo "📋 Send this ZIP to your Windows friend. They just need to:"
echo "1. Install Rust from rustup.rs"
echo "2. Run build_windows.bat"
echo "3. Enjoy the app!"
