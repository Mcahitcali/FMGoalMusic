# Tesseract Windows Bundle

This directory contains the Tesseract OCR files needed for Windows distribution.

## Required Files (to be added manually):

### Core DLLs (download from Tesseract Windows release):
- `tesseract.dll` - Main Tesseract library
- `liblept176.dll` - Image processing library

### Language Data:
- `tessdata/eng.traineddata` ✅ (already downloaded)

## Where to get the DLLs:

1. Download Tesseract Windows installer from: https://github.com/UB-Mannheim/tesseract/releases
2. Extract the installer using 7-Zip or similar tool
3. Copy these files to this directory:
   - From `tesseract-ocr/tesseract.exe` directory: `tesseract.dll`
   - From same directory: `liblept176.dll`

## Alternative - Direct Download:
You can also download the portable ZIP and extract the DLLs from there.

## Final Structure:
```
assets/tesseract/
├── tesseract.dll
├── liblept176.dll
└── tessdata/
    └── eng.traineddata
```

The build script will automatically include these files in the Windows distribution.
