# FM Goal Musics - Build and Distribution Guide

This guide explains how to build distributable packages for FM Goal Musics on different platforms.

## Quick Start

### macOS
```bash
# Build macOS app bundle and DMG
./build_macos.sh

# Or use the cross-platform script
./build.sh
```

### Windows
```batch
# Standard build (portable app)
build_windows.bat

# Smart Installer (recommended for distribution)
build_windows_installer.bat

# Or use the cross-platform script (if available)
build.sh
```

### Windows Installer Features
The smart installer (`build_windows_installer.bat`) includes:
- **Professional installation wizard** with license agreement
- **Automatic Tesseract OCR detection** and installation guidance
- **Start menu and desktop shortcuts**
- **Proper uninstaller** with registry cleanup
- **Component selection** (app, Tesseract, shortcuts)
- **Windows version compatibility check**

### Linux
```bash
# Build Linux binary and AppImage
./build.sh
```

## Requirements

### Common Requirements
- Rust 1.70+ with Cargo
- All project dependencies from Cargo.toml

### Platform-Specific Requirements

#### macOS
- Xcode Command Line Tools
- `hdiutil` (included with macOS)
- Optional: Icon Composer for creating .icns files

#### Windows
- Visual Studio Build Tools or Visual Studio Community
- MSVC toolchain
- Optional: Inno Setup for creating installers

#### Linux
- Basic build tools (gcc, make)
- Optional: `appimagetool` for creating AppImages

## Build Outputs

### macOS
- `build/macos/FM Goal Musics.app` - Application bundle
- `build/macos/FM Goal Musics-YYYYMMDD.dmg` - Distributable DMG

### Windows
- `build/windows/FM-Goal-Musics-Setup-1.0.0.exe` - Professional installer (recommended)
- `build/windows/fm-goal-musics-gui.exe` - Main executable (portable)
- `build/windows/FM Goal Musics-Windows-Portable-YYYYMMDD.zip` - Portable ZIP
- `build/windows/README.txt` - Installation and troubleshooting guide

### Linux
- `build/linux/fm-goal-musics-gui` - Main executable
- `build/linux/FM Goal Musics.AppDir/` - AppImage structure
- `build/linux-FM Goal Musics-YYYYMMDD.tar.gz` - Distributable TAR

## Customization

### Application Icon
Replace the placeholder icon in `assets/`:
- macOS: `assets/icon.icns` (512x512 recommended)
- Windows: `assets/icon.ico` (256x256 recommended)
- Linux: `assets/icon.png` (256x256 recommended)

### Application Metadata
Edit the following files to customize application metadata:
- `build_macos.sh` - Update APP_NAME and BUNDLE_ID
- `build_windows.bat` - Update APP_NAME
- `build.sh` - Update APP_NAME and VERSION

## Distribution

### macOS
1. Distribute the DMG file
2. Users can drag the app to their Applications folder
3. First-time users may need to right-click and "Open" to bypass Gatekeeper

### Windows (Recommended)
1. **Distribute the installer**: `FM-Goal-Musics-Setup-1.0.0.exe`
2. Users run the installer - it handles everything automatically
3. **Tesseract OCR**: Auto-detected and installation guidance provided
4. Creates shortcuts and proper uninstaller

### Windows (Portable Alternative)
1. Distribute the ZIP file
2. Users extract and run the executable
3. Manual Tesseract OCR installation required for text recognition

## Troubleshooting

### Build Failures
- Ensure all dependencies are installed
- Check that you have the correct toolchain for your target platform
- Run `cargo clean` and try again

### Permission Issues (macOS/Linux)
```bash
chmod +x build.sh
chmod +x build_macos.sh
```

### Missing Icons
The build will work without icons, but users should add proper icon files to `assets/` for professional distribution.

## Advanced Options

### Cross-Compilation
For building on one platform for another:
```bash
# Add target toolchain
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-apple-darwin

# Build for specific target
cargo build --release --target x86_64-pc-windows-msvc
```

### Custom Build Profiles
Create custom build profiles in `Cargo.toml` for different distribution needs (debug builds, etc.).

### Automated Builds
Consider integrating with CI/CD systems like GitHub Actions for automated builds and releases.
