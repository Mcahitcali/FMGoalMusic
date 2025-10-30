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
# Build Windows executable and ZIP
build_windows.bat

# Or use the cross-platform script (if available)
build.sh
```

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
- `build/windows/fm-goal-musics-gui.exe` - Main executable
- `build/windows/FM Goal Musics-Windows-YYYYMMDD.zip` - Distributable ZIP

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

### Windows
1. Distribute the ZIP file
2. Users extract and run the executable
3. No installation required (portable app)

### Linux
1. Distribute the TAR file
2. Users extract and run the binary
3. Or create an AppImage for better distribution

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
