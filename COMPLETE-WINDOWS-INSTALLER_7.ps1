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

# Step 2.5: Install Tesseract OCR (system-wide, optional when using vcpkg)
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

# Step 2.6: Configure native dependencies (prefer system dev files, fallback to vcpkg)
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "STEP 2.6: Native dependencies (system or vcpkg)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$UsingVcpkg = $false

# Detect system development files
$sysRoot = "C:\Program Files\Tesseract-OCR"
$sysIncludeRoot = Join-Path $sysRoot "include"
$sysLibRoot = Join-Path $sysRoot "lib"
$hasHeaders = (Test-Path (Join-Path $sysIncludeRoot "leptonica\allheaders.h")) -and (Test-Path (Join-Path $sysIncludeRoot "tesseract\capi.h"))
$hasLibs = (Test-Path (Join-Path $sysLibRoot "leptonica.lib")) -and (Test-Path (Join-Path $sysLibRoot "tesseract.lib"))

if ($hasHeaders -and $hasLibs) {
    Write-Host "Found system Tesseract/Leptonica development files. Using system libraries." -ForegroundColor Green
    $env:LEPT_NO_PKG_CONFIG = "1"
    $env:TESS_NO_PKG_CONFIG = "1"
    $env:LEPTONICA_INCLUDE_PATH = $sysIncludeRoot
    $env:LEPTONICA_LINK_PATHS   = $sysLibRoot
    $env:LEPTONICA_LINK_LIBS    = "leptonica"
    $env:TESSERACT_INCLUDE_PATH = $sysIncludeRoot
    $env:TESSERACT_LINK_PATHS   = $sysLibRoot
    $env:TESSERACT_LINK_LIBS    = "tesseract"
    # Ensure runtime DLLs available
    if (Test-Path $sysRoot) { $env:PATH += ";$sysRoot" }
    # Avoid stale vcpkg settings impacting build
    Remove-Item Env:VCPKG_ROOT -ErrorAction SilentlyContinue
    Remove-Item Env:VCPKG_DEFAULT_TRIPLET -ErrorAction SilentlyContinue
} else {
    Write-Host "System development files not found. Falling back to vcpkg installation." -ForegroundColor Yellow
    $UsingVcpkg = $true

    # Determine vcpkg path
    $defaultVcpkgPaths = @(
        "C:\\vcpkg",
        (Join-Path $env:USERPROFILE "vcpkg")
    )

    $vcpkgRoot = $null
    foreach ($p in $defaultVcpkgPaths) {
        if (Test-Path (Join-Path $p "vcpkg.exe")) { $vcpkgRoot = $p; break }
        if (Test-Path $p) { $vcpkgRoot = $p }
    }
    if (-not $vcpkgRoot) { $vcpkgRoot = $defaultVcpkgPaths[0] }

    if (-not (Test-Path (Join-Path $vcpkgRoot "vcpkg.exe"))) {
        Write-Host "Installing vcpkg at $vcpkgRoot ..." -ForegroundColor Blue
        try {
            if (-not (Test-Path $vcpkgRoot)) { New-Item -ItemType Directory -Path $vcpkgRoot | Out-Null }
            # Clone vcpkg
            $gitExe = "git"
            Push-Location $vcpkgRoot
            if (-not (Test-Path (Join-Path $vcpkgRoot ".git"))) {
                & $gitExe clone https://github.com/microsoft/vcpkg.git . 2>&1 | Out-Null
            }
            # Bootstrap
            if (Test-Path (Join-Path $vcpkgRoot "bootstrap-vcpkg.bat")) {
                & (Join-Path $vcpkgRoot "bootstrap-vcpkg.bat") -disableMetrics 2>&1 | Write-Verbose
            }
            Pop-Location
            Write-Host "vcpkg installed successfully" -ForegroundColor Green
        } catch {
            Pop-Location 2>$null
            Write-Host "Failed to install vcpkg" -ForegroundColor Red
            Write-Host $_.Exception.Message
            Write-Host "Press any key to exit..."
            $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
            exit 1
        }
    }

    # Ensure env for vcpkg is set so Rust build scripts can find it
    $env:VCPKG_ROOT = $vcpkgRoot
    $env:PATH += ";" + (Join-Path $vcpkgRoot "")
    $env:VCPKG_DEFAULT_TRIPLET = "x64-windows"

    # Do not force pkg-config bypass when using vcpkg
    Remove-Item Env:LEPT_NO_PKG_CONFIG -ErrorAction SilentlyContinue
    Remove-Item Env:TESS_NO_PKG_CONFIG -ErrorAction SilentlyContinue

    # Install required packages
    Write-Host "Installing native dependencies via vcpkg (this may take a while)..." -ForegroundColor Blue
    try {
        & (Join-Path $vcpkgRoot "vcpkg.exe") install tesseract:x64-windows 2>&1 | Write-Verbose
        # leptonica is a dependency of tesseract; ensure installed explicitly as well
        & (Join-Path $vcpkgRoot "vcpkg.exe") install leptonica:x64-windows 2>&1 | Write-Verbose
        Write-Host "vcpkg dependencies installed" -ForegroundColor Green
    } catch {
        Write-Host "vcpkg package installation failed" -ForegroundColor Red
        Write-Host $_.Exception.Message
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
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

# Re-assert vcpkg environment after VS setup (VsDevCmd may override VCPKG_ROOT)
if ($UsingVcpkg) {
    if (-not $vcpkgRoot -or -not (Test-Path (Join-Path $vcpkgRoot "vcpkg.exe"))) {
        $vcpkgRoot = "C:\vcpkg"
    }
    $env:VCPKG_ROOT = $vcpkgRoot
    # Force dynamic triplet to match installed ports and avoid static-md mismatch
    $env:VCPKG_DEFAULT_TRIPLET = "x64-windows"
    $env:VCPKGRS_TRIPLET = "x64-windows"
    $env:VCPKGRS_DYNAMIC = "1"
    Remove-Item Env:VCPKGRS_STATIC -ErrorAction SilentlyContinue
    $env:PATH += ";" + (Join-Path $vcpkgRoot "")
}

# Ensure libclang is available for bindgen (required by leptonica-sys)
function Test-LibClangPath($dir) {
    return (Test-Path (Join-Path $dir "libclang.dll")) -or (Test-Path (Join-Path $dir "clang.dll"))
}

if (-not $env:LIBCLANG_PATH -or -not (Test-LibClangPath $env:LIBCLANG_PATH)) {
    Write-Host "Checking for libclang (required by bindgen)..." -ForegroundColor Blue
    $candidateDirs = @(
        "C:\\Program Files\\LLVM\\bin",
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\Llvm\\x64\\bin",
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\Llvm\\bin"
    )
    $found = $false
    foreach ($dir in $candidateDirs) {
        if (Test-LibClangPath $dir) { $env:LIBCLANG_PATH = $dir; $found = $true; break }
    }
    if (-not $found) {
        Write-Host "libclang not found; attempting to install LLVM via winget..." -ForegroundColor Yellow
        try {
            $winget = (Get-Command winget -ErrorAction SilentlyContinue)
            if ($winget) {
                & winget install -e --id LLVM.LLVM --accept-package-agreements --accept-source-agreements --silent
                $llvmDir = "C:\\Program Files\\LLVM\\bin"
                if (Test-LibClangPath $llvmDir) {
                    $env:LIBCLANG_PATH = $llvmDir
                    Write-Host "Configured LIBCLANG_PATH=$llvmDir" -ForegroundColor Green
                    # Add to PATH for runtime if needed
                    $env:PATH += ";$llvmDir"
                    $found = $true
                }
            } else {
                Write-Host "winget not available; please install LLVM or ensure libclang.dll is present." -ForegroundColor Yellow
            }
        } catch {
            Write-Host "Failed to install LLVM via winget: $($_.Exception.Message)" -ForegroundColor Yellow
        }
    }
    if (-not $found) {
        Write-Host "libclang still not found. You can install LLVM from https://llvm.org or via Visual Studio 'Clang/LLVM for MSBuild'." -ForegroundColor Red
        Write-Host "Set LIBCLANG_PATH to the folder containing libclang.dll and re-run the installer." -ForegroundColor Red
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
}
Write-Host "Building application..." -ForegroundColor Blue
Write-Host "   This will take 10-15 minutes on first build..." -ForegroundColor Yellow
Write-Host "   Please be patient - this is normal!" -ForegroundColor Yellow
Write-Host ""

# Summarize dependency configuration for build
if ($UsingVcpkg) {
    Write-Host "Configured to use vcpkg dependencies (VCPKG_ROOT=$env:VCPKG_ROOT, triplet=$env:VCPKG_DEFAULT_TRIPLET)" -ForegroundColor Green
} else {
    Write-Host "Configured to use system Tesseract/Leptonica development files at '$sysRoot'" -ForegroundColor Green
}

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
