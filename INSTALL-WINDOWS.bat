@echo off
REM FM Goal Musics - One-Click Windows Installer
REM This script installs Rust and builds the application automatically

setlocal enabledelayedexpansion

echo.
echo ========================================
echo   FM Goal Musics - Auto Installer
echo ========================================
echo.
echo This installer will:
echo 1. Install Rust (if not installed)
echo 2. Build the application
echo 3. Create ready-to-use package
echo.

REM Check if Rust is installed
echo 🔍 Checking for Rust installation...
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ Rust not found. Installing Rust...
    echo.
    echo 📥 Downloading Rust installer...
    
    REM Download Rust installer
    powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile 'rustup-init.exe'"
    
    if not exist "rustup-init.exe" (
        echo ❌ Failed to download Rust installer
        pause
        exit /b 1
    )
    
    echo 📦 Installing Rust...
    echo This may take a few minutes...
    
    REM Install Rust silently with default options
    start /wait "" rustup-init.exe -y --default-toolchain stable
    
    REM Clean up installer
    del rustup-init.exe
    
    REM Refresh PATH
    call "%USERPROFILE%\.cargo\env.bat"
    
    echo ✅ Rust installed successfully!
) else (
    echo ✅ Rust is already installed
)

REM Refresh environment variables
echo 🔄 Refreshing environment...
call "%USERPROFILE%\.cargo\env.bat" 2>nul
set PATH=%PATH%;%USERPROFILE%\.cargo\bin

REM Verify Rust is working
echo 🔧 Verifying Rust installation...
rustc --version
if %errorlevel% neq 0 (
    echo ❌ Rust installation verification failed
    echo Please restart your computer and run this installer again
    pause
    exit /b 1
)

echo.
echo 🏗️  Building FM Goal Musics...
echo This may take 10-15 minutes on first build...
echo.

REM Run the build script
if exist "build_windows.bat" (
    call build_windows.bat
    if %errorlevel% neq 0 (
        echo ❌ Build failed
        pause
        exit /b 1
    )
) else (
    echo ❌ build_windows.bat not found
    pause
    exit /b 1
)

echo.
echo ========================================
echo   ✅ INSTALLATION COMPLETE!
echo ========================================
echo.
echo 📍 Your application is ready in:
echo    build\windows\fm-goal-musics-gui.exe
echo.
echo 🎮 To run the application:
echo    1. Navigate to build\windows\
echo    2. Double-click fm-goal-musics-gui.exe
echo.
echo 📦 Or share the ZIP file with others:
echo    build\windows\FM-Goal-Musics-Windows-*.zip
echo.
echo 🎉 Enjoy your goal celebration music!
echo.

REM Ask if user wants to run the app
set /p choice="Do you want to run FM Goal Musics now? (Y/N): "
if /i "%choice%"=="Y" (
    echo 🚀 Starting FM Goal Musics...
    if exist "build\windows\fm-goal-musics-gui.exe" (
        start "" "build\windows\fm-goal-musics-gui.exe"
    ) else (
        echo ❌ Application not found at build\windows\fm-goal-musics-gui.exe
    )
)

echo.
echo Press any key to exit...
pause >nul
