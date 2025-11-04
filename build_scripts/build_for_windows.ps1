<#
.SYNOPSIS
  Prepare Windows environment, build FM Goal Musics, and stage a self-contained payload for NSIS.
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
        & $Test  # re-check
    } else {
        Write-Host "  Found $Name"
    }
}

# ----------------------------- #
#  0. TOOLCHAIN & NATIVE DEPS
# ----------------------------- #
Ensure-Choco

# Visual C++ toolchain (msvc), CMake, pkg-config, Tesseract, NSIS
Ensure-Tool -Name "MSVC Build Tools" -Test { cmd /c cl /? | Out-Null } -ChocoPkg "visualstudio2022buildtools"
Ensure-Tool -Name "CMake"            -Test { cmake --version  | Out-Null } -ChocoPkg "cmake"
Ensure-Tool -Name "pkg-config"       -Test { pkg-config --version | Out-Null } -ChocoPkg "pkgconfiglite"
Ensure-Tool -Name "Tesseract OCR"    -Test { tesseract --version | Out-Null } -ChocoPkg "tesseract"
Ensure-Tool -Name "NSIS"             -Test { makensis /VERSION  | Out-Null } -ChocoPkg "nsis"

# Rust toolchain is installed by the workflow step; just verify
Write-Host "Checking Rust toolchain..."
rustc --version
cargo --version

# ----------------------------- #
#  1. PATHS & CLEAN
# ----------------------------- #
$scriptPath = $MyInvocation.MyCommand.Definition
$projectRoot = Split-Path -Parent $scriptPath
$repoRoot = Resolve-Path (Join-Path $projectRoot "..")
$buildDir = Join-Path $repoRoot "build/windows"
$exeName = "fm-goal-musics-gui.exe"

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

# Optional project assets if they exist
$maybeAssets = @("config", "assets", "README.md", "LICENSE")
foreach ($item in $maybeAssets) {
    $src = Join-Path $repoRoot $item
    if (Test-Path $src) {
        Copy-Item $src -Destination $buildDir -Recurse -Force
        Write-Host "  Included: $item"
    }
}

# Bundle Tesseract so app works on a clean machine
$tessSrc = Join-Path ${env:ProgramFiles} "Tesseract-OCR"
if (Test-Path $tessSrc) {
    Copy-Item $tessSrc -Destination (Join-Path $buildDir "Tesseract-OCR") -Recurse -Force
    Write-Host "  Included: Tesseract-OCR"
} else {
    Write-Warning "Tesseract-OCR install not found. The installer will lack OCR runtime!"
}

# ----------------------------- #
#  4. DONE
# ----------------------------- #
Write-Host "[3/3] Build staging complete." -ForegroundColor Green
Write-Host "Payload: $buildDir"
Write-Host "Binary:  $exeName"
