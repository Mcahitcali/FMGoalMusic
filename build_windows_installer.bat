@echo off
REM FM Goal Musics - Enhanced Windows Build Script with Smart Installer
REM Creates a professional installer with Tesseract OCR detection

echo 🪟 Building FM Goal Musics for Windows with Smart Installer...

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

REM Copy icon if exists
if exist "assets\icon.ico" (
    echo 🎨 Copying application icon...
    copy "assets\icon.ico" "%BUILD_DIR%\"
)

REM Copy default ambiance sound
if exist "goal_crowd_cheer.wav" (
    echo 🎵 Copying default ambiance sound...
    copy "goal_crowd_cheer.wav" "%BUILD_DIR%\"
)

REM Create a comprehensive README
echo 📄 Creating README...
echo %APP_NAME% > "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo Version 1.0.0 >> "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo Installation Instructions: >> "%BUILD_DIR%\README.txt"
echo ========================= >> "%BUILD_DIR%\README.txt"
echo 1. Run FM-Goal-Musics-Setup.exe to install with all components >> "%BUILD_DIR%\README.txt"
echo 2. The installer will automatically detect and install Tesseract OCR if needed >> "%BUILD_DIR%\README.txt"
echo 3. Launch from Start Menu or Desktop shortcut >> "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo Portable Mode (Advanced Users): >> "%BUILD_DIR%\README.txt"
echo ================================= >> "%BUILD_DIR%\README.txt"
echo If you prefer portable mode, you can run fm-goal-musics-gui.exe directly >> "%BUILD_DIR%\README.txt"
echo but you'll need to install Tesseract OCR manually for text recognition. >> "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo Troubleshooting: >> "%BUILD_DIR%\README.txt"
echo =============== >> "%BUILD_DIR%\README.txt"
echo - If text recognition doesn't work, install Tesseract OCR from: >> "%BUILD_DIR%\README.txt"
echo   https://github.com/UB-Mannheim/tesseract/wiki >> "%BUILD_DIR%\README.txt"
echo - Make sure Football Manager is running before starting detection >> "%BUILD_DIR%\README.txt"
echo - Check Windows security settings if screen capture fails >> "%BUILD_DIR%\README.txt"
echo. >> "%BUILD_DIR%\README.txt"
echo For more information and updates, visit the project repository. >> "%BUILD_DIR%\README.txt%"

REM Create a batch file for easy running (portable mode)
echo 🚀 Creating launcher...
echo @echo off > "%BUILD_DIR%\Run FM Goal Musics.bat"
echo echo Starting FM Goal Musics... >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo echo. >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo echo Checking for Tesseract OCR... >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo where tesseract >nul 2>&1 >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo if errorlevel 1 ( >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo     echo ⚠️  Tesseract OCR not found! Text recognition will not work. >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo     echo Please install Tesseract OCR from: >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo     echo https://github.com/UB-Mannheim/tesseract/wiki >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo     echo. >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo ) >> "%BUILD_DIR%\Run FM Goal Musics.bat"
echo start "" "%BINARY_NAME%.exe" >> "%BUILD_DIR%\Run FM Goal Musics.bat"

REM Check if NSIS is available for installer creation
echo 📦 Checking for NSIS installer...
where makensis >nul 2>&1
if errorlevel 1 (
    echo ⚠️  NSIS not found. Creating ZIP package only.
    echo To create an installer, install NSIS from: https://nsis.sourceforge.io/
    goto create_zip
)

REM Create NSIS installer
echo 🏗️  Creating professional installer...
copy "installer.nsi" "%INSTALLER_DIR%\"
copy "LICENSE.txt" "%INSTALLER_DIR%\"

REM Update NSIS script with correct paths
echo Generating installer script...
echo !define BUILDPATH "%CD%\%BUILD_DIR%" > "%INSTALLER_DIR%\installer_generated.nsi"
type "installer.nsi" >> "%INSTALLER_DIR%\installer_generated.nsi"

REM Build the installer
cd "%INSTALLER_DIR%"
makensis "installer_generated.nsi"
if errorlevel 1 (
    echo ❌ Installer creation failed!
    cd ..\..
    goto create_zip
)
cd ..\..

echo ✅ Professional installer created successfully!
echo 📍 Installer: %INSTALLER_DIR%\FM-Goal-Musics-Setup-1.0.0.exe

:create_zip
REM Create ZIP archive for portable distribution
echo 📦 Creating ZIP archive...
powershell -command "Compress-Archive -Path '%BUILD_DIR%\*' -DestinationPath '%BUILD_DIR%\%APP_NAME%-Windows-Portable-$(Get-Date -Format 'yyyyMMdd').zip' -Force"

echo ✅ Enhanced Windows build completed successfully!
echo 📍 Build directory: %BUILD_DIR%
echo.
echo 📦 Distribution Files:
if exist "%INSTALLER_DIR%\FM-Goal-Musics-Setup-1.0.0.exe" (
    echo    🏗️  Professional Installer: %INSTALLER_DIR%\FM-Goal-Musics-Setup-1.0.0.exe
    echo    ✨ Recommended for most users - includes auto Tesseract detection
)
echo    📱 Portable ZIP: %BUILD_DIR%\%APP_NAME%-Windows-Portable-*.zip
echo.
echo 🎯 Installation Options:
echo    📦 Installer: Double-click FM-Goal-Musics-Setup.exe (recommended)
echo    📱 Portable: Extract ZIP and run fm-goal-musics-gui.exe
echo.
echo The installer will automatically detect and guide users through Tesseract OCR installation!
pause
