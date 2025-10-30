#!/bin/bash

# Download Tesseract Windows DLLs for bundling
# This script downloads the necessary Tesseract files for Windows distribution

set -e

TESSERACT_DIR="assets/tesseract"
echo "🔤 Downloading Tesseract Windows files for bundling..."

# Create directory structure
mkdir -p "$TESSERACT_DIR/tessdata"

# Download English language data (if not already present)
if [ ! -f "$TESSERACT_DIR/tessdata/eng.traineddata" ]; then
    echo "📥 Downloading English language data..."
    curl -L -o "$TESSERACT_DIR/tessdata/eng.traineddata" \
        "https://github.com/tesseract-ocr/tessdata/raw/main/eng.traineddata"
else
    echo "✅ English language data already exists"
fi

# Download Tesseract Windows portable ZIP
echo "📥 Downloading Tesseract Windows portable..."
curl -L -o "$TESSERACT_DIR/tesseract-portable.zip" \
    "https://github.com/UB-Mannheim/tesseract/releases/download/5.3.3.20231005/tesseract-ocr-w64-setup-5.3.3.20231005.exe"

# Extract DLLs from the installer
echo "📦 Extracting Tesseract DLLs..."
if command -v 7z &> /dev/null; then
    # Use 7-Zip if available
    7z e "$TESSERACT_DIR/tesseract-portable.zip" -o"$TESSERACT_DIR/" \
        "tesseract-ocr/tesseract.dll" "tesseract-ocr/liblept176.dll" 2>/dev/null || {
        echo "⚠️  Could not extract DLLs automatically"
        echo "Please manually extract tesseract.dll and liblept176.dll from the installer"
        echo "and place them in $TESSERACT_DIR/"
    }
else
    echo "⚠️  7-Zip not found for automatic extraction"
    echo "Please manually extract tesseract.dll and liblept176.dll from:"
    echo "  $TESSERACT_DIR/tesseract-portable.zip"
    echo "And place them in: $TESSERACT_DIR/"
fi

# Clean up
rm -f "$TESSERACT_DIR/tesseract-portable.zip"

# Check final structure
echo "🔍 Checking Tesseract bundle structure..."
if [ -f "$TESSERACT_DIR/tesseract.dll" ] && [ -f "$TESSERACT_DIR/liblept176.dll" ]; then
    echo "✅ Tesseract bundle ready!"
    echo "📁 Contents:"
    ls -la "$TESSERACT_DIR/"
    echo "📁 tessdata:"
    ls -la "$TESSERACT_DIR/tessdata/"
else
    echo "⚠️  Tesseract DLLs missing - please add them manually"
    echo "See $TESSERACT_DIR/README.md for instructions"
fi
