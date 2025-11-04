<#
.SYNOPSIS
  Prepare Windows environment (incl. vcpkg), build FM Goal Musics, and stage a self-contained payload for NSIS.
#>

$ErrorActionPreference = "Stop"
Write-Host "========== FM Goal Musics - Windows Setup & Build ==========" -ForegroundColor Cyan

function Ensure-Choco {
    if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Host "Installing Chocolatey..."
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12
        iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    }
}

function Ensure-Tool {
    param(
        [string]$Name,
        [scriptblock]$Test,
        [string]$ChocoPkg
    )
    Write-Host "Checking $Name..."
    $ok = $false
    try { & $Test; $ok = $true } catch { $ok = $false }
    if (-not $ok) {
        Write-Host "Installing $Name..."
        choco install $ChocoPkg -y --no-progress
    } else {
        Write-Host "  Found $Name"
    }
}

# ----------------------------- #
#  0. TOOLCHAIN & NATIVE DEPS
# ----------------------------- #
Ensure-Choco
Ensure-Tool -Name "MSVC Build Tools" -Test { cmd /c cl /? | Out-Null }  -ChocoPkg "visualstudio2022buildtools"
Ensure-Tool -Name "CMake"            -Test { cmake --version  | Out-Null } -ChocoPkg "cmake"
Ensure-Tool -Name "pkg-config"       -Test { pkg-config --version | Out-Null } -ChocoPkg "pkgconfiglite"
Ensure-Tool -Name "NSIS"             -Test { makensis /VERSION  | Out-Null }   -ChocoPkg "nsis"

# Rust toolchain is installed by the workflow step
Write-Host "Checking Rust toolchain..."
rustc --version
cargo --version

# ----------------------------- #
#  vcpkg (headers + libs for leptonica/tesseract)
# ----------------------------- #
$userHome  = $env:USERPROFILE  # <- renamed to avoid clobbering $HOME
$vcpkgRoot = Join-Path $userHome "vcpkg"
$vcpkgExe  = Join-Path $vcpkgRoot "vcpkg.exe"

if (-not (Test-Path $vcpkgExe)) {
    Write-Host "Installing vcpkg..."
    git clone --depth=1 https://github.com/microsoft/vcpkg $vcpkgRoot | Out-Null
    & (Join-Path $vcpkgRoot "bootstrap-vcpkg.bat") -disableMetrics | Out-Null
}

# Configure vcpkg for x64 Windows and dynamic linking
$env:VCPKG_ROOT             = $vcpkgRoot
$env:VCPKGRS_TRIPLET        = "x64-windows"
$env:VCPKGRS_DYNAMIC        = "1"
$env:VCPKG_DEFAULT_TRIPLET  = "x64-windows"

Write-Host "Installing vcpkg ports: leptonica:x64-windows, tesseract:x64-windows ..."
& $vcpkgExe install leptonica:x64-windows tesseract:x64-windows --clean-after-build | Out-Null

# Make Tesseract CLI visible in-session if present
$tessDir  = Join-Path ${env:ProgramFiles} "Tesseract-OCR"
$tessExe  = Join-Path $tessDir "tesseract.exe"
if (Test-Path $tessExe) {
    if (-not (($env:Path -split ';') -contains $tessDir)) {
        $env:Path = "$tessDir;$env:Path"
        Write-Host "Session PATH updated with: $tessDir"
    }
    if (-not $env:TESSDATA_PREFIX) {
        $env:TESSDATA_PREFIX = $tessDir
        Write-Host "Session TESSDATA_PREFIX set to: $env:TESSDATA_PREFIX"
    }
} else {
    Write-Host "Chocolatey Tesseract not found; relying on vcpkg dev libs only (OK for build)."
}

# ----------------------------- #
#  1. PATHS & CLEAN
# ----------------------------- #
$scriptPath = $MyInvocation.MyCommand.Definition
$projectRoot = Split-Path -Parent $scriptPath
$repoRoot = Resolve-Path (Join-Path $projectRoot "..")
$buildDir = Join-Path $repoRoot "build/windows"
$exeName  = "fm-goal-musics-gui.exe"

if (Test-Path $buildDir) { Remove-Item -Recurse -Force $buildDir }
New-Item -ItemType Directory -Force -Path $buildDir | Out-Null

# ----------------------------- #
#  2. BUILD
# ----------------------------- #
Write-Host "[1/3] Building Rust (release, msvc)..." -ForegroundColor Yellow
Set-Location $repoRoot
cargo build --release --target x86_64-pc-windows-msvc

$binaryPath = Join-Path $repoRoot "target/x86_64-pc-windows-msvc/release/$exeName"
if (!(Test-Path $binaryPath)) {
    throw "Build failed: $exeName not found: $binaryPath"
}
Copy-Item $binaryPath -Destination $buildDir

# ----------------------------- #
#  3. PACKAGE RUNTIME FILES
# ----------------------------- #
Write-Host "[2/3] Staging runtime files..." -ForegroundColor Yellow

$maybeAssets = @("config", "assets", "README.md", "LICENSE")
foreach ($item in $maybeAssets) {
    $src = Join-Path $repoRoot $item
    if (Test-Path $src) {
        Copy-Item $src -Destination $buildDir -Recurse -Force
        Write-Host "  Included: $item"
    }
}

# Bundle vcpkg-built runtime DLLs so the app runs on clean machines
$vcpkgBin = Join-Path $vcpkgRoot "installed\x64-windows\bin"
if (Test-Path $vcpkgBin) {
    Copy-Item (Join-Path $vcpkgBin "*.dll") -Destination $buildDir -Force -ErrorAction SilentlyContinue
    Write-Host "  Included: vcpkg runtime DLLs from $vcpkgBin"
} else {
    Write-Warning "vcpkg bin folder not found; dynamic DLLs may be missing at runtime."
}

# Bundle Chocolatey Tesseract runtime as well (handy for CLI use and tessdata)
if (Test-Path $tessDir) {
    Copy-Item $tessDir -Destination (Join-Path $buildDir "Tesseract-OCR") -Recurse -Force
    Write-Host "  Included: Tesseract-OCR runtime"
}

# ----------------------------- #
#  4. DONE
# ----------------------------- #
Write-Host "[3/3] Build staging complete." -ForegroundColor Green
Write-Host "Payload: $buildDir"
Write-Host "Binary:  $exeName"
