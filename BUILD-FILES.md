# Build Files - Clean Structure

## 📁 Essential Build Files

### 🍎 **macOS Development**
- `build_macos.sh` - Build macOS .app bundle and DMG
- `build.sh` - Cross-platform build script

### 🪟 **Windows Distribution** 
- `build_windows.bat` - Windows build script (for building on Windows)
- `INSTALL-WINDOWS.bat` - One-click installer for Windows users
- `INSTALL-WINDOWS.ps1` - PowerShell one-click installer (recommended)

### 📦 **Distribution Package**
- `FM-Goal-Musics-Windows-Source-*.zip` - Complete Windows package (auto-generated)

## 🎯 **What to Share with Windows Users:**

**Send them:** `FM-Goal-Musics-Windows-Source-*.zip`

**Inside the ZIP, users just:**
1. Right-click `INSTALL-WINDOWS.ps1`
2. Select "Run with PowerShell"
3. Everything happens automatically!

## 🗑️ **Removed Redundant Files:**

- `build_macos_enhanced.sh` - Duplicate of build_macos.sh
- `build_installer.bat` - NSIS builder (complex)
- `installer.nsi`/`installer.iss` - Professional installer scripts
- `download_tesseract.sh` - Helper script
- `create_windows_package.sh` - Package creation script
- `INSTALLER-README.md` - Duplicate documentation
- `LICENSE.txt` - Installer license

## 🚀 **Simple & Clean:**

Now you have exactly what you need:
- **Development scripts** for building
- **One-click installers** for users
- **No confusion** with too many options
