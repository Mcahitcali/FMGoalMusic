# FM Goal Musics - Complete Windows Installer
# This ONE file installs EVERYTHING needed and builds the project
# Just right-click and "Run with PowerShell"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  FM Goal Musics - Complete Installer" -ForegroundColor Cyan  
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This installer will automatically:" -ForegroundColor Yellow
Write-Host "1. Install Visual Studio Build Tools (C++ compiler)" -ForegroundColor White
Write-Host "2. Install Rust toolchain" -ForegroundColor White
Write-Host "3. Build FM Goal Musics application" -ForegroundColor White
Write-Host "4. Create desktop shortcut" -ForegroundColor White
Write-Host ""
Write-Host "Total time: 20-30 minutes (mostly automatic)" -ForegroundColor Yellow
Write-Host "Disk space needed: ~5 GB" -ForegroundColor Yellow
Write-Host ""

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")
if (-NOT $isAdmin) {
    Write-Host "This installer needs administrator privileges" -ForegroundColor Red
    Write-Host "Please right-click this file and select 'Run as Administrator'" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

Write-Host "Running with administrator privileges" -ForegroundColor Green
Write-Host ""

# Step 1: Install Visual Studio Build Tools
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "STEP 1: Visual Studio Build Tools" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Visual Studio Build Tools are already installed
$vsInstalled = Test-Path "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools"
if (-not $vsInstalled) {
    $vsInstalled = Test-Path "C:\Program Files\Microsoft Visual Studio\2022\BuildTools"
}

if ($vsInstalled) {
    Write-Host "Visual Studio Build Tools already installed" -ForegroundColor Green
} else {
    Write-Host "Downloading Visual Studio Build Tools..." -ForegroundColor Blue
    Write-Host "   (This is a large download: ~1.5 GB)" -ForegroundColor Yellow
    
    $vsInstallerPath = Join-Path $env:TEMP "vs_buildtools.exe"
    try {
        Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile $vsInstallerPath
        Write-Host "Download completed" -ForegroundColor Green
    } catch {
        Write-Host "Failed to download Visual Studio Build Tools" -ForegroundColor Red
        Write-Host $_.Exception.Message
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    Write-Host ""
    Write-Host "Installing Visual Studio Build Tools..." -ForegroundColor Blue
    Write-Host "   This will take 10-15 minutes..." -ForegroundColor Yellow
    Write-Host "   A separate installer window will open - please wait for it to complete" -ForegroundColor Yellow
    Write-Host ""
    
    # Install with required components
    $vsArgs = @(
        "--quiet",
        "--wait",
        "--norestart",
        "--nocache",
        "--add", "Microsoft.VisualStudio.Workload.VCTools",
        "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
        "--add", "Microsoft.VisualStudio.Component.Windows11SDK.22000"
    )
    
    try {
        Start-Process -FilePath $vsInstallerPath -ArgumentList $vsArgs -Wait -NoNewWindow
        Write-Host "Visual Studio Build Tools installed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "Visual Studio Build Tools installation failed" -ForegroundColor Red
        Write-Host $_.Exception.Message
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    # Clean up installer
    Remove-Item $vsInstallerPath -Force -ErrorAction SilentlyContinue
}

Write-Host ""

# Step 2: Install Rust
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "STEP 2: Rust Toolchain" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Rust is installed
$cargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (Test-Path $cargoPath) {
    $rustVersion = & $cargoPath --version
    Write-Host "Rust is already installed: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "Downloading Rust installer..." -ForegroundColor Blue
    
    $rustInstallerPath = Join-Path $env:TEMP "rustup-init.exe"
    try {
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustInstallerPath
        Write-Host "Download completed" -ForegroundColor Green
    } catch {
        Write-Host "Failed to download Rust installer" -ForegroundColor Red
        Write-Host $_.Exception.Message
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    Write-Host "Installing Rust..." -ForegroundColor Blue
    Write-Host "   This may take 2-5 minutes..." -ForegroundColor Yellow
    
    try {
        Start-Process -FilePath $rustInstallerPath -ArgumentList "-y", "--default-toolchain", "stable" -Wait -NoNewWindow
        Write-Host "Rust installed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "Rust installation failed" -ForegroundColor Red
        Write-Host $_.Exception.Message
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    # Clean up installer
    Remove-Item $rustInstallerPath -Force -ErrorAction SilentlyContinue
    
    # Update cargo path
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
}

Write-Host ""

# Verify Rust is working
Write-Host "Verifying Rust installation..." -ForegroundColor Blue
try {
    $rustcPath = Join-Path $env:USERPROFILE ".cargo\bin\rustc.exe"
    if (Test-Path $rustcPath) {
        $rustVersion = & $rustcPath --version
        Write-Host "Rust verification successful: $rustVersion" -ForegroundColor Green
    } else {
        throw "Rust compiler not found"
    }
} catch {
    Write-Host "Rust verification failed" -ForegroundColor Red
    Write-Host "Please restart your computer and run this installer again" -ForegroundColor Yellow
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

Write-Host ""

# Step 2.5: Install Tesseract OCR
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "STEP 2.5: Tesseract OCR" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if Tesseract is installed
$tesseractInstalled = Test-Path "C:\Program Files\Tesseract-OCR\tesseract.exe"
if (-not $tesseractInstalled) {
    $tesseractInstalled = Test-Path "C:\Program Files (x86)\Tesseract-OCR\tesseract.exe"
}

if ($tesseractInstalled) {
    Write-Host "Tesseract OCR already installed" -ForegroundColor Green
} else {
    Write-Host "Downloading Tesseract OCR..." -ForegroundColor Blue
    Write-Host "   (Download size: ~50 MB)" -ForegroundColor Yellow
    
    $tesseractInstallerPath = Join-Path $env:TEMP "tesseract-installer.exe"
    try {
        Invoke-WebRequest -Uri "https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-5.3.3.20231005.exe" -OutFile $tesseractInstallerPath
        Write-Host "Download completed" -ForegroundColor Green
    } catch {
        Write-Host "Failed to download Tesseract OCR" -ForegroundColor Red
        Write-Host $_.Exception.Message
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    Write-Host "Installing Tesseract OCR..." -ForegroundColor Blue
    Write-Host "   This will take 1-2 minutes..." -ForegroundColor Yellow
    
    try {
        Start-Process -FilePath $tesseractInstallerPath -ArgumentList "/S" -Wait -NoNewWindow
        Write-Host "Tesseract OCR installed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "Tesseract OCR installation failed" -ForegroundColor Red
        Write-Host $_.Exception.Message
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    # Clean up installer
    Remove-Item $tesseractInstallerPath -Force -ErrorAction SilentlyContinue
    
    # Add Tesseract to PATH
    $tesseractPath = "C:\Program Files\Tesseract-OCR"
    if (Test-Path $tesseractPath) {
        $env:PATH += ";$tesseractPath"
        Write-Host "Added Tesseract to PATH" -ForegroundColor Green
    }
}

Write-Host ""

# Step 3: Build the application
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "STEP 3: Building FM Goal Musics" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Setup Visual Studio environment
Write-Host "Setting up Visual Studio environment..." -ForegroundColor Blue

# Find Visual Studio installation
$vsPaths = @(
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat"
)

$vsDevCmd = $null
foreach ($path in $vsPaths) {
    if (Test-Path $path) {
        $vsDevCmd = $path
        break
    }
}

if ($vsDevCmd) {
    Write-Host "Found Visual Studio at: $vsDevCmd" -ForegroundColor Green
    
    # Create a temporary batch file to capture environment variables
    $tempBat = Join-Path $env:TEMP "setup_vs_env.bat"
    $tempEnv = Join-Path $env:TEMP "vs_env.txt"
    
    # Create batch file that sets up VS environment and exports variables
    @"
@echo off
call "$vsDevCmd" -arch=x64 -host_arch=x64 >nul
set > "$tempEnv"
"@ | Out-File -FilePath $tempBat -Encoding ASCII
    
    # Run the batch file
    cmd /c $tempBat
    
    # Read the environment variables
    if (Test-Path $tempEnv) {
        Get-Content $tempEnv | ForEach-Object {
            if ($_ -match '^([^=]+)=(.*)$') {
                $name = $matches[1]
                $value = $matches[2]
                Set-Item -Path "env:$name" -Value $value -ErrorAction SilentlyContinue
            }
        }
        Remove-Item $tempEnv -Force -ErrorAction SilentlyContinue
    }
    
    Remove-Item $tempBat -Force -ErrorAction SilentlyContinue
    Write-Host "Visual Studio environment configured" -ForegroundColor Green
} else {
    Write-Host "Warning: Could not find Visual Studio Developer Command Prompt" -ForegroundColor Yellow
    Write-Host "Build may fail if C++ tools are not in PATH" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Building application..." -ForegroundColor Blue
Write-Host "   This will take 10-15 minutes on first build..." -ForegroundColor Yellow
Write-Host "   Please be patient - this is normal!" -ForegroundColor Yellow
Write-Host ""

# Set environment variables to skip vcpkg and use system libraries
$env:LEPT_NO_PKG_CONFIG = "1"
$env:TESS_NO_PKG_CONFIG = "1"
Write-Host "Configured to skip vcpkg dependency checks" -ForegroundColor Green

# Build the project
try {
    $buildOutput = & $cargoPath build --release --bin fm-goal-musics-gui 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        Write-Host ""
        Write-Host "Error details:" -ForegroundColor Yellow
        Write-Host $buildOutput
        Write-Host ""
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    Write-Host "Build completed successfully!" -ForegroundColor Green
} catch {
    Write-Host "Build failed with exception" -ForegroundColor Red
    Write-Host $_.Exception.Message
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

Write-Host ""

# Step 4: Create distribution
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "STEP 4: Creating Distribution" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$buildDir = "build\windows"
$exeName = "fm-goal-musics-gui.exe"

# Create build directory
New-Item -ItemType Directory -Force -Path $buildDir | Out-Null

# Copy executable
Write-Host "Copying executable..." -ForegroundColor Blue
Copy-Item "target\release\$exeName" "$buildDir\" -Force

# Copy assets
if (Test-Path "assets") {
    Write-Host "Copying assets..." -ForegroundColor Blue
    Copy-Item -Path "assets" -Destination "$buildDir\" -Recurse -Force
}

# Copy default sound
if (Test-Path "goal_crowd_cheer.wav") {
    Write-Host "Copying default sound..." -ForegroundColor Blue
    Copy-Item "goal_crowd_cheer.wav" "$buildDir\" -Force
}

# Create desktop shortcut
Write-Host "Creating desktop shortcut..." -ForegroundColor Blue
$WshShell = New-Object -ComObject WScript.Shell
$desktopPath = [System.Environment]::GetFolderPath('Desktop')
$shortcutPath = Join-Path $desktopPath "FM Goal Musics.lnk"
$Shortcut = $WshShell.CreateShortcut($shortcutPath)
$Shortcut.TargetPath = Join-Path $PWD "$buildDir\$exeName"
$Shortcut.WorkingDirectory = Join-Path $PWD $buildDir
$Shortcut.IconLocation = Join-Path $PWD "$buildDir\$exeName"
$Shortcut.Description = "FM Goal Musics - Goal celebration music player"
$Shortcut.Save()

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  INSTALLATION COMPLETE!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Your application is ready at:" -ForegroundColor Yellow
$fullPath = Join-Path $PWD "$buildDir\$exeName"
Write-Host "   $fullPath" -ForegroundColor White
Write-Host ""
Write-Host "To run the application:" -ForegroundColor Yellow
Write-Host "   - Double-click the desktop shortcut: 'FM Goal Musics'" -ForegroundColor White
Write-Host "   - Or navigate to: $buildDir\" -ForegroundColor White
Write-Host ""
Write-Host "Enjoy your goal celebration music!" -ForegroundColor Green
Write-Host ""

# Ask if user wants to run the app
$choice = Read-Host "Do you want to run FM Goal Musics now? (Y/N)"
if ($choice -eq "Y" -or $choice -eq "y") {
    Write-Host "Starting FM Goal Musics..." -ForegroundColor Blue
    $exePath = Join-Path $PWD "$buildDir\$exeName"
    Start-Process -FilePath $exePath
}

Write-Host ""
Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
