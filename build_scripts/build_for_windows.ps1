[CmdletBinding(DefaultParameterSetName = "default")]
param (
    [Parameter(ParameterSetName = "setup")]
    [switch] $SetupOnly,

    [Parameter(ParameterSetName = "build")]
    [switch] $BuildOnly,

    [switch] $SkipDistribution
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Write-Heading {
    param(
        [string] $Title
    )

    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host " $Title" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
}

function Test-IsAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Ensure-Administrator { # only when we must install
    param(
        [bool] $RequireAdmin
    )

    if (-not $RequireAdmin) {
        return
    }

    if (Test-IsAdministrator) {
        Write-Host "Running with administrator privileges" -ForegroundColor Green
        return
    }

    Write-Host "Administrator privileges are required to install missing dependencies." -ForegroundColor Red
    Write-Host "Please right-click this file and choose 'Run as Administrator'." -ForegroundColor Yellow
    throw "Insufficient privileges"
}

$script:CargoPath = $null
$script:UsingVcpkg = $false
$script:VcpkgRoot = $null
$script:VsDevCmd = $null

function Ensure-VisualStudioBuildTools {
    Write-Heading "Step 1: Visual Studio Build Tools"

    $installPaths = @(
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools",
        "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools"
    )

    foreach ($path in $installPaths) {
        if (Test-Path $path) {
            Write-Host "Visual Studio Build Tools already installed at $path" -ForegroundColor Green
            return
        }
    }

    Ensure-Administrator -RequireAdmin $true

    $vsInstallerPath = Join-Path $env:TEMP "vs_buildtools.exe"
    Write-Host "Downloading Visual Studio Build Tools... (~1.5 GB)" -ForegroundColor Blue
    Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile $vsInstallerPath
    Write-Host "Installing Visual Studio Build Tools..." -ForegroundColor Blue

    $args = @(
        "--quiet",
        "--wait",
        "--norestart",
        "--nocache",
        "--add", "Microsoft.VisualStudio.Workload.VCTools",
        "--add", "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
        "--add", "Microsoft.VisualStudio.Component.Windows11SDK.22000"
    )

    Start-Process -FilePath $vsInstallerPath -ArgumentList $args -Wait -NoNewWindow
    Write-Host "Visual Studio Build Tools installed successfully" -ForegroundColor Green
    Remove-Item $vsInstallerPath -Force -ErrorAction SilentlyContinue
}

function Ensure-RustToolchain {
    Write-Heading "Step 2: Rust Toolchain"

    $script:CargoPath = Join-Path $env:USERPROFILE ".cargo\\bin\\cargo.exe"
    if (Test-Path $script:CargoPath) {
        $version = & $script:CargoPath --version
        Write-Host "Rust already installed: $version" -ForegroundColor Green
        return
    }

    Ensure-Administrator -RequireAdmin $true

    $rustInstaller = Join-Path $env:TEMP "rustup-init.exe"
    Write-Host "Downloading Rust installer..." -ForegroundColor Blue
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustInstaller
    Write-Host "Installing Rust (stable toolchain)..." -ForegroundColor Blue
    Start-Process -FilePath $rustInstaller -ArgumentList "-y", "--default-toolchain", "stable" -Wait -NoNewWindow
    Remove-Item $rustInstaller -Force -ErrorAction SilentlyContinue

    $script:CargoPath = Join-Path $env:USERPROFILE ".cargo\\bin\\cargo.exe"
    $rustc = Join-Path $env:USERPROFILE ".cargo\\bin\\rustc.exe"
    if (-not (Test-Path $rustc)) {
        throw "Rust installation failed: rustc.exe not found"
    }

    Write-Host "Rust installed successfully" -ForegroundColor Green
}

function Get-TesseractRoot {
    $paths = @(
        "C:\\Program Files\\Tesseract-OCR",
        "C:\\Program Files (x86)\\Tesseract-OCR"
    )

    foreach ($path in $paths) {
        if (Test-Path (Join-Path $path "tesseract.exe")) {
            return $path
        }
    }

    return $null
}

function Ensure-TesseractRuntime {
    Write-Heading "Step 3: Tesseract OCR Runtime"

    $existing = Get-TesseractRoot
    if ($existing) {
        Write-Host "Tesseract already installed at $existing" -ForegroundColor Green
        return $existing
    }

    Ensure-Administrator -RequireAdmin $true

    $installer = Join-Path $env:TEMP "tesseract-installer.exe"
    Write-Host "Downloading Tesseract OCR (≈50 MB)..." -ForegroundColor Blue
    Invoke-WebRequest -Uri "https://digi.bib.uni-mannheim.de/tesseract/tesseract-ocr-w64-setup-5.3.3.20231005.exe" -OutFile $installer
    Write-Host "Installing Tesseract OCR..." -ForegroundColor Blue
    Start-Process -FilePath $installer -ArgumentList "/S" -Wait -NoNewWindow
    Remove-Item $installer -Force -ErrorAction SilentlyContinue

    $installed = Get-TesseractRoot
    if ($installed) {
        Write-Host "Tesseract installed at $installed" -ForegroundColor Green
        return $installed
    }

    throw "Tesseract installation did not succeed"
}

function Ensure-NativeDevFiles {
    param(
        [string] $TesseractRoot
    )

    Write-Heading "Step 4: Native Dependencies"

    $includeRoot = Join-Path $TesseractRoot "include"
    $libRoot = Join-Path $TesseractRoot "lib"
    $hasHeaders = (Test-Path (Join-Path $includeRoot "leptonica\\allheaders.h")) -and (Test-Path (Join-Path $includeRoot "tesseract\\capi.h"))
    $hasLibs = (Test-Path (Join-Path $libRoot "leptonica.lib")) -and (Test-Path (Join-Path $libRoot "tesseract.lib"))

    if ($hasHeaders -and $hasLibs) {
        Write-Host "Using system Tesseract development files" -ForegroundColor Green
        $env:LEPT_NO_PKG_CONFIG = "1"
        $env:TESS_NO_PKG_CONFIG = "1"
        $env:LEPTONICA_INCLUDE_PATH = $includeRoot
        $env:LEPTONICA_LINK_PATHS = $libRoot
        $env:LEPTONICA_LINK_LIBS = "leptonica"
        $env:TESSERACT_INCLUDE_PATH = $includeRoot
        $env:TESSERACT_LINK_PATHS = $libRoot
        $env:TESSERACT_LINK_LIBS = "tesseract"
        if (Test-Path $TesseractRoot) {
            $env:PATH += ";$TesseractRoot"
        }
        $script:UsingVcpkg = $false
        return
    }

    Write-Host "System development files not found. Falling back to vcpkg." -ForegroundColor Yellow
    $script:UsingVcpkg = $true

    $defaultRoots = @(
        "C:\\vcpkg",
        (Join-Path $env:USERPROFILE "vcpkg")
    )

    foreach ($candidate in $defaultRoots) {
        if (Test-Path (Join-Path $candidate "vcpkg.exe")) {
            $script:VcpkgRoot = $candidate
            break
        }
    }

    if (-not $script:VcpkgRoot) {
        $script:VcpkgRoot = $defaultRoots[0]
    }

    if (-not (Test-Path (Join-Path $script:VcpkgRoot "vcpkg.exe"))) {
        Ensure-Administrator -RequireAdmin $true
        Write-Host "Installing vcpkg at $script:VcpkgRoot" -ForegroundColor Blue
        if (-not (Test-Path $script:VcpkgRoot)) {
            New-Item -ItemType Directory -Force -Path $script:VcpkgRoot | Out-Null
        }
        Push-Location $script:VcpkgRoot
        try {
            if (-not (Test-Path (Join-Path $script:VcpkgRoot ".git"))) {
                & git clone https://github.com/microsoft/vcpkg.git . | Out-Null
            }
            & (Join-Path $script:VcpkgRoot "bootstrap-vcpkg.bat") -disableMetrics | Out-Null
        } finally {
            Pop-Location
        }
    }

    $env:VCPKG_ROOT = $script:VcpkgRoot
    $env:VCPKG_DEFAULT_TRIPLET = "x64-windows"
    $env:VCPKGRS_TRIPLET = "x64-windows"
    $env:VCPKGRS_DYNAMIC = "1"
    Remove-Item Env:VCPKGRS_STATIC -ErrorAction SilentlyContinue
    $env:PATH += ";" + (Join-Path $script:VcpkgRoot "")

    Write-Host "Installing native dependencies via vcpkg (tesseract, leptonica)..." -ForegroundColor Blue
    & (Join-Path $script:VcpkgRoot "vcpkg.exe") install tesseract:x64-windows | Write-Verbose
    & (Join-Path $script:VcpkgRoot "vcpkg.exe") install leptonica:x64-windows | Write-Verbose
    Write-Host "vcpkg dependencies installed" -ForegroundColor Green
}

function Configure-VisualStudioEnvironment {
    Write-Heading "Step 5: Configure Visual Studio Environment"

    $vsCmdCandidates = @(
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\Common7\\Tools\\VsDevCmd.bat",
        "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\Common7\\Tools\\VsDevCmd.bat",
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\Common7\\Tools\\VsDevCmd.bat",
        "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\Community\\Common7\\Tools\\VsDevCmd.bat"
    )

    foreach ($candidate in $vsCmdCandidates) {
        if (Test-Path $candidate) {
            $script:VsDevCmd = $candidate
            break
        }
    }

    if (-not $script:VsDevCmd) {
        Write-Host "Warning: Could not find VsDevCmd.bat. Continuing, but build may fail." -ForegroundColor Yellow
        return
    }

    Write-Host "Using Visual Studio environment script at $script:VsDevCmd" -ForegroundColor Green

    $tempBat = Join-Path $env:TEMP "setup_vs_env.bat"
    $tempEnv = Join-Path $env:TEMP "vs_env.txt"

    @"
@echo off
call "$script:VsDevCmd" -arch=x64 -host_arch=x64 >nul
set > "$tempEnv"
"@ | Out-File -FilePath $tempBat -Encoding ASCII

    cmd /c $tempBat | Out-Null

    if (Test-Path $tempEnv) {
        Get-Content $tempEnv | ForEach-Object {
            if ($_ -match '^([^=]+)=(.*)$') {
                Set-Item -Path ("env:" + $matches[1]) -Value $matches[2] -ErrorAction SilentlyContinue
            }
        }
        Remove-Item $tempEnv -Force -ErrorAction SilentlyContinue
    }

    Remove-Item $tempBat -Force -ErrorAction SilentlyContinue
    Write-Host "Visual Studio environment configured" -ForegroundColor Green

    if ($script:UsingVcpkg -and $script:VcpkgRoot) {
        $env:VCPKG_ROOT = $script:VcpkgRoot
        $env:VCPKG_DEFAULT_TRIPLET = "x64-windows"
        $env:VCPKGRS_TRIPLET = "x64-windows"
        $env:VCPKGRS_DYNAMIC = "1"
        $env:PATH += ";" + (Join-Path $script:VcpkgRoot "")
    }
}

function Ensure-LibClang {
    Write-Heading "Step 6: Ensure libclang for bindgen"

    function Test-LibClangPath([string] $dir) {
        return (Test-Path (Join-Path $dir "libclang.dll")) -or (Test-Path (Join-Path $dir "clang.dll"))
    }

    if ($env:LIBCLANG_PATH -and (Test-LibClangPath $env:LIBCLANG_PATH)) {
        Write-Host "LIBCLANG_PATH already set to $env:LIBCLANG_PATH" -ForegroundColor Green
        return
    }

    $candidateDirs = @(
        "C:\\Program Files\\LLVM\\bin",
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\Llvm\\x64\\bin",
        "C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\Llvm\\bin"
    )

    foreach ($dir in $candidateDirs) {
        if (Test-LibClangPath $dir)) {
            $env:LIBCLANG_PATH = $dir
            Write-Host "Configured LIBCLANG_PATH=$dir" -ForegroundColor Green
            return
        }
    }

    Write-Host "libclang not found. Attempting to install LLVM via winget..." -ForegroundColor Yellow
    $winget = Get-Command winget -ErrorAction SilentlyContinue
    if ($winget) {
        Ensure-Administrator -RequireAdmin $true
        & winget install -e --id LLVM.LLVM --accept-package-agreements --accept-source-agreements --silent
        $llvmDir = "C:\\Program Files\\LLVM\\bin"
        if (Test-LibClangPath $llvmDir) {
            $env:LIBCLANG_PATH = $llvmDir
            $env:PATH += ";$llvmDir"
            Write-Host "Configured LIBCLANG_PATH=$llvmDir" -ForegroundColor Green
            return
        }
    } else {
        Write-Host "winget not available. Please install LLVM manually." -ForegroundColor Yellow
    }

    throw "libclang not available. Install LLVM and set LIBCLANG_PATH manually."
}

function Invoke-CargoBuild {
    Write-Heading "Step 7: Build FM Goal Musics"

    if (-not (Test-Path $script:CargoPath)) {
        throw "cargo.exe not found. Ensure Rust toolchain is installed."
    }

    Write-Host "Running cargo build --release --bin fm-goal-musics-gui" -ForegroundColor Blue
    $output = & $script:CargoPath build --release --bin fm-goal-musics-gui 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host $output
        throw "Cargo build failed"
    }
    Write-Host "Build completed successfully" -ForegroundColor Green
}

function Prepare-Distribution {
    Write-Heading "Step 8: Prepare Distribution Folder"

    $projectRoot = Get-Location
    $releaseDir = Join-Path $projectRoot "target\release"
    $exeName = "fm-goal-musics-gui.exe"
    $exePath = Join-Path $releaseDir $exeName

    if (-not (Test-Path $exePath)) {
        throw "Expected executable not found at $exePath"
    }

    $outputDir = Join-Path $projectRoot "build\windows"
    New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

    Write-Host "Copying executable to $outputDir" -ForegroundColor Blue
    Copy-Item $exePath (Join-Path $outputDir $exeName) -Force

    $folders = @("config", "assets", "tessdata")
    foreach ($folder in $folders) {
        $source = Join-Path $projectRoot $folder
        if (Test-Path $source) {
            Write-Host "Copying $folder" -ForegroundColor Blue
            $target = Join-Path $outputDir $folder
            New-Item -ItemType Directory -Force -Path $target | Out-Null
            Copy-Item (Join-Path $source '*') $target -Recurse -Force
        } else {
            Write-Host "$folder folder not found, skipping" -ForegroundColor Yellow
        }
    }

    $defaultSound = Join-Path $projectRoot "goal_crowd_cheer.wav"
    if (Test-Path $defaultSound) {
        Copy-Item $defaultSound (Join-Path $outputDir "goal_crowd_cheer.wav") -Force
    }

    Write-Host "Distribution folder ready at $outputDir" -ForegroundColor Green
    Write-Host "You can now build the NSIS installer via FMGoalMusicInstaller.nsi" -ForegroundColor Yellow
}

function Ensure-ProjectTessdata {
    param(
        [string] $TesseractRoot
    )

    $projectRoot = Get-Location
    $projectTessdata = Join-Path $projectRoot "tessdata"
    if (Test-Path $projectTessdata) {
        Write-Host "Project tessdata directory already present" -ForegroundColor Green
        return
    }

    $systemTessdata = Join-Path $TesseractRoot "tessdata"
    if (Test-Path $systemTessdata) {
        Write-Host "Copying tessdata from system Tesseract to project" -ForegroundColor Blue
        New-Item -ItemType Directory -Force -Path $projectTessdata | Out-Null
        Copy-Item (Join-Path $systemTessdata '*') $projectTessdata -Recurse -Force
        return
    }

    Write-Host "Warning: tessdata not found in project or system. OCR may fail." -ForegroundColor Yellow
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  FM Goal Musics – Windows Setup & Build" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$doSetup = -not $BuildOnly
$doBuild = -not $SetupOnly
$doDistribution = $doBuild -and -not $SkipDistribution

try {
    $tesseractRoot = $null

    if ($doSetup) {
        Ensure-VisualStudioBuildTools
        Ensure-RustToolchain
        $tesseractRoot = Ensure-TesseractRuntime
        Ensure-NativeDevFiles -TesseractRoot $tesseractRoot
    } else {
        $script:CargoPath = Join-Path $env:USERPROFILE ".cargo\\bin\\cargo.exe"
        $tesseractRoot = ("C:\\Program Files\\Tesseract-OCR", "C:\\Program Files (x86)\\Tesseract-OCR" | Where-Object { Test-Path (Join-Path $_ "tesseract.exe") })[0]
    }

    if (-not $tesseractRoot) {
        Write-Host "Warning: Tesseract root could not be determined. Continuing." -ForegroundColor Yellow
    }

    if ($doBuild) {
        if (-not $script:CargoPath) {
            $script:CargoPath = Join-Path $env:USERPROFILE ".cargo\\bin\\cargo.exe"
        }
        Configure-VisualStudioEnvironment
        Ensure-LibClang
        if ($tesseractRoot) {
            Ensure-ProjectTessdata -TesseractRoot $tesseractRoot
        }
        Invoke-CargoBuild
    }

    if ($doDistribution) {
        Prepare-Distribution
    }

    Write-Host "All requested steps completed successfully" -ForegroundColor Green
} catch {
    Write-Host ""; Write-Host "❌ Script failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}