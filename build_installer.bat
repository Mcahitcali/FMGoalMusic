@echo off
REM Build FM Goal Musics Windows Installer using NSIS

echo ========================================
echo   FM Goal Musics - Installer Builder
echo ========================================
echo.

REM Check if NSIS is installed
echo 🔍 Checking for NSIS installation...
where makensis >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ NSIS not found. Please install NSIS first:
    echo    Download from: https://nsis.sourceforge.io/Download
    echo.
    echo After installing NSIS, run this script again.
    pause
    exit /b 1
)

echo ✅ NSIS found

REM Check if icon exists
if not exist "assets\icon.ico" (
    echo ⚠️  Warning: assets\icon.ico not found
    echo Installer will use default icon
)

REM Build the installer
echo.
echo 🔨 Building Windows installer...
echo This may take a few minutes...
echo.

makensis "installer.nsi"

if %errorlevel% neq 0 (
    echo ❌ Installer build failed
    pause
    exit /b 1
)

echo.
echo ========================================
echo   ✅ INSTALLER BUILT SUCCESSFULLY!
echo ========================================
echo.
echo 📍 Your installer is ready:
echo    FM-Goal-Musics-Installer.exe
echo.
echo 📦 What users get:
echo    • Professional graphical installer
echo    • Automatic Rust installation
echo    • No terminal windows visible
echo    • Desktop and Start Menu shortcuts
echo    • Built-in uninstaller
echo.
echo 🎉 Ready to distribute!
echo.

REM Ask if user wants to test the installer
set /p choice="Do you want to test the installer now? (Y/N): "
if /i "%choice%"=="Y" (
    echo 🚀 Starting installer test...
    start "" "FM-Goal-Musics-Installer.exe"
)

echo.
echo Press any key to exit...
pause >nul
