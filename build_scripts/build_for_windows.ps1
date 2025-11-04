<#
.SYNOPSIS
  Prepare Windows environment (MSVC + vcpkg), build FM Goal Musics, and stage a self-contained payload for NSIS.
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

# Import environment variables from a batch file into the current PowerShell process
function Import-BatchEnv {
    param([string]$BatchCmd)

    $tmp = New-TemporaryFile
    $cmd = "`"$BatchCmd`" && set"
    cmd.exe /c $cmd > $tmp 2>&1
    $exit = $LASTEXITCODE
    $out = Get-Content $tmp -Raw
    if ($exit -ne 0) {
        Remove-Item $tmp -Force
        throw "Failed to import environment via: $BatchCmd`n$out"
    }

    Get-Content $tmp | ForEach-Object {
        if ($_ -match '^(.*?)=(.*)$') {
            $name  = $matches[1]
            $value = $matches[2]

            if ($name -ieq 'HOME' -or [string]::IsNullOrWhiteSpace($name)) { return }

            try {
                Set-Item -Path ("Env:{0}" -f $name) -Value $value -ErrorAction Stop
            } catch {
                [System.Environment]::SetEnvironmentVariable($name, $value, 'Process')
            }
        }
    }

    Remove-Item $tmp -Force
}

function Find-VsDevCmd {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) {
        Ensure-Choco
        choco install vswhere -y --no-progress | Out-Null
    }
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) { return $null }

    $installationPath = & $vswhere -latest -products * -requires Microsoft.Component.MSBuild Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $installationPath) { return $null }

    $devCmd = Join-Path $installationPath "Common7\Tools\VsDevCmd.bat"
    if (Test-Path $devCmd) { return $devCmd }
    return $null
}

function Ensure-Msvc-OnPath {
    $cl = Get-Command cl.exe -ErrorAction SilentlyContinue
    $link = Get-Command link.exe -ErrorAction SilentlyContinue
    if ($cl -and $link) { Write-Host "MSVC is available on PATH."; return }

    $devCmd = Find-VsDevCmd
    if ($devCmd) {
        Write-Host "Importing VS developer environment..."
        Import-BatchEnv "`"$devCmd`" -host_arch=amd64 -arch=amd64"
        $cl = Get-Command cl.exe -ErrorAction SilentlyContinue
        $link = Get-Command link.exe -ErrorAction SilentlyContinue
        if ($cl -and $link) { Write-Host "MSVC imported successfully."; return }
    }

    Write-Host "Installing Visual Studio 2022 Build Tools (C++ toolchain)..."
    Ensure-Choco
    choco install visualstudio2022buildtools -y --no-progress | Out-Null

    $devCmd = Find-VsDevCmd
    if (-not $devCmd) { throw "VsDevCmd.bat not found even after installing Build Tools." }

    Write-Host "Importing VS developer environment after installation..."
    Import-BatchEnv "`"$devCmd`" -host_arch=amd64 -arch=amd64"

    $cl = Get-Command cl.exe -ErrorAction SilentlyContinue
    $link = Get-Command link.exe -ErrorAction SilentlyContinue
    if (-not ($cl -and $link)) { throw "MSVC not available on PATH after VsDevCmd import." }
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
        Ensure-Choco
        choco install $ChocoPkg -y --no-progress | Out-Null
        & $Test
        Write-Host "  Installed $Name"
    } else {
        Write-Host "  Found $Name"
    }
}

# ----------------------------- #
# 0) Toolchain & prerequisites
# ----------------------------- #
Ensure-Msvc-OnPath
Ensure-Tool -Name "CMake"      -Test { cmake --version  | Out-Null }        -ChocoPkg "cmake"
Ensure-Tool -Name "pkg-config" -Test { pkg-config --version | Out-Null }    -ChocoPkg "pkgconfiglite"
Ensure-Tool -Name "NSIS"       -Test { makensis /VERSION  | Out-Null }      -ChocoPkg "nsis"

Write-Host "Checking Rust toolchain (from workflow)..."
rustc --version
cargo --version

# ----------------------------- #
# 1) vcpkg (leptonica + tesseract)
# ----------------------------- #
$userHome  = $env:USERPROFILE
$vcpkgRoot = Join-Path $userHome "vcpkg"
$vcpkgExe  = Join-Path $vcpkgRoot "vcpkg.exe"

if (-not (Test-Path $vcpkgExe)) {
    Write-Host "Installing vcpkg..."
    git clone --depth=1 https://github.com/microsoft/vcpkg $vcpkgRoot | Out-Null
    & (Join-Path $vcpkgRoot "bootstrap-vcpkg.bat") -disableMetrics | Out-Null
}

$env:VCPKG_ROOT                 = $vcpkgRoot
$env:VCPKG_DEFAULT_TRIPLET      = "x64-windows"
$env:VCPKGRS_TRIPLET            = "x64-windows"
$env:VCPKGRS_DYNAMIC            = "1"
$env:VCPKG_FEATURE_FLAGS        = "manifests,binarycaching"
$env:CMAKE_BUILD_PARALLEL_LEVEL = "2"
$env:VCPKG_MAX_CONCURRENCY      = "2"

$binaryCache = Join-Path $vcpkgRoot "binarycache"
if (!(Test-Path $binaryCache)) { New-Item -ItemType Directory -Path $binaryCache | Out-Null }
$env:VCPKG_DEFAULT_BINARY_CACHE = $binaryCache

Write-Host "Installing vcpkg ports (leptonica, tesseract) with binary cache..."
& $vcpkgExe install leptonica:x64-windows tesseract:x64-windows --binarysource=("clear;files={0},readwrite" -f $binaryCache) --clean-after-build | Out-Null

# ----------------------------- #
# 2) Build
# ----------------------------- #
$scriptPath = $MyInvocation.MyCommand.Definition
$projectRoot = Split-Path -Parent $scriptPath
$repoRoot = Resolve-Path (Join-Path $projectRoot "..")
$buildDir = Join-Path $repoRoot "build/windows"
$exeName  = "fm-goal-musics-gui.exe"

if (Test-Path $buildDir) { Remove-Item -Recurse -Force $buildDir }
New-Item -ItemType Directory -Force -Path $buildDir | Out-Null

Write-Host "[1/3] Building Rust (release, MSVC)..." -ForegroundColor Yellow
Set-Location $repoRoot
cargo build --release --target x86_64-pc-windows-msvc

$binaryPath = Join-Path $repoRoot "target/x86_64-pc-windows-msvc/release/$exeName"
if (!(Test-Path $binaryPath)) {
    throw "Build failed: $exeName not found: $binaryPath"
}
Copy-Item $binaryPath -Destination $buildDir

# ----------------------------- #
# 3) Stage runtime files
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

# vcpkg DLLs (leptonica, tesseract, deps)
$vcpkgBin = Join-Path $vcpkgRoot "installed\x64-windows\bin"
if (Test-Path $vcpkgBin) {
    Copy-Item (Join-Path $vcpkgBin "*.dll") -Destination $buildDir -Force -ErrorAction SilentlyContinue
    Write-Host "  Included: vcpkg runtime DLLs from $vcpkgBin"
} else {
    Write-Warning "vcpkg bin folder not found; runtime DLLs may be missing."
}

# tessdata (OCR languages)
$tessdataSrc = Join-Path $vcpkgRoot "installed\x64-windows\share\tesseract\tessdata"
if (Test-Path $tessdataSrc) {
    Copy-Item $tessdataSrc -Destination (Join-Path $buildDir "tessdata") -Recurse -Force
    Write-Host "  Included: tessdata"
} else {
    Write-Warning "tessdata not found in vcpkg; OCR may miss language data."
}

Write-Host "[3/3] Build staging complete." -ForegroundColor Green
Write-Host "Payload: $buildDir"
Write-Host "Binary:  $exeName"
