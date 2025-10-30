@echo off
REM FM Goal Musics - Windows Build Script
REM Creates a distributable .exe with installer for Windows

echo 🪟 Building FM Goal Musics for Windows...

REM Configuration
set APP_NAME=FM Goal Musics
set BINARY_NAME=fm-goal-musics-gui
set SOURCE_DIR=src
set TARGET_DIR=target\release
set BUILD_DIR=build\windows
set INSTALLER_DIR=%BUILD_DIR%\installer

REM Clean previous builds
echo 🧹 Cleaning previous builds...
if exist "%BUILD_DIR%" rmdir /s /q "%BUILD_DIR%"
mkdir "%BUILD_DIR%"
mkdir "%INSTALLER_DIR%"

REM Build the release binary
echo 🔨 Building release binary...
cargo build --release --bin %BINARY_NAME% --target x86_64-pc-windows-msvc

REM Check if build was successful
if not exist "%TARGET_DIR%\%BINARY_NAME%.exe" (
    echo ❌ Build failed! Binary not found.
    pause
    exit /b 1
)

REM Copy binary and dependencies
echo 📋 Copying binary and dependencies...
copy "%TARGET_DIR%\%BINARY_NAME%.exe" "%BUILD_DIR%\"

REM Copy necessary DLLs and resources
echo 📚 Copying resources...
if exist "assets" xcopy /E /I "assets" "%BUILD_DIR%\assets\"

REM Create a simple README
echo 📄 Creating README...
echo %APP_NAME% > "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo Installation Instructions: >> "%BUILD_DIR%\README.txt"
echo 1. Double-click FM-Goal-Musics-gui.exe to run the application >> "%BUILD_DIR%\README.txt"
echo 2. No installation required - portable application >> "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo For more information, visit the project repository. >> "%BUILD_DIR%\README.txt%"

REM Create a batch file for easy running
echo 🚀 Creating launcher...
echo @echo off > "%BUILD_DIR%\Run FM Goal Musics.bat"
echo echo Starting FM Goal Musics... >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo start "" "%BINARY_NAME%.exe" >> "%BUILD_DIR%\Run FM Goal Musics.bat"

REM Create ZIP archive
echo 📦 Creating ZIP archive...
powershell -command "Compress-Archive -Path '%BUILD_DIR%\*' -DestinationPath '%BUILD_DIR%\%APP_NAME%-Windows-$(Get-Date -Format 'yyyyMMdd').zip' -Force"

echo ✅ Windows build completed successfully!
echo 📍 Build directory: %BUILD_DIR%
echo 📦 ZIP file created in: %BUILD_DIR%
echo.
echo To test the app:
echo 1. Navigate to %BUILD_DIR%
echo 2. Double-click "%BINARY_NAME%.exe" or "Run FM Goal Musics.bat"
echo.
echo Distribution files are ready in %BUILD_DIR%
pause
