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

function Import-BatchEnv {
    param(
        [Parameter(Mandatory=$true)][string]$BatchFile,
        [Parameter(Mandatory=$false)][string]$Args = ""
    )

    if (-not (Test-Path $BatchFile)) {
        throw "Batch file not found: $BatchFile"
    }

    $tmp = New-TemporaryFile
    $cmdLine = "`"$BatchFile`" $Args && set"
    cmd.exe /c $cmdLine > $tmp 2>&1
    $exit = $LASTEXITCODE
    $out = Get-Content $tmp -Raw
    if ($exit -ne 0) {
        Remove-Item $tmp -Force
        throw "Failed to import environment via:`n$cmdLine`n$out"
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
        Import-BatchEnv -BatchFile $devCmd -Args "-host_arch=amd64 -arch=amd64"
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
    Import-BatchEnv -BatchFile $devCmd -Args "-host_arch=amd64 -arch=amd64"

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
        try {
            & $Test
            Write-Host "  Installed $Name"
        } catch {
            Write-Warning "$Name installed but not yet on PATH; attempting path resolution."
            return $false
        }
    } else {
        Write-Host "  Found $Name"
    }
    return $true
}

function Resolve-Makensis {
    $cmd = Get-Command makensis.exe -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Path }

    $candidates = @(
        "${env:ProgramFiles(x86)}\NSIS\makensis.exe",
        "${env:ProgramFiles}\NSIS\makensis.exe"
    )
    foreach ($p in $candidates) {
        if (Test-Path $p) {
            $dir = Split-Path $p -Parent
            if (-not (($env:Path -split ';') -contains $dir)) {
                $env:Path = "$dir;$env:Path"
            }
            return $p
        }
    }
    return $null
}

# ----------------------------- #
# 0) Toolchain & prerequisites
# ----------------------------- #
Ensure-Msvc-OnPath
Ensure-Tool -Name "CMake"      -Test { cmake --version  | Out-Null }        -ChocoPkg "cmake"
Ensure-Tool -Name "pkg-config" -Test { pkg-config --version | Out-Null }    -ChocoPkg "pkgconfiglite"

# NSIS: install, then resolve absolute path, then verify
$nsisOk = Ensure-Tool -Name "NSIS" -Test { makensis /VERSION | Out-Null } -ChocoPkg "nsis"
$makensisPath = Resolve-Makensis
if (-not $makensisPath) {
    throw "NSIS makensis.exe not found after installation. Looked under Program Files\NSIS. Check Chocolatey logs."
} else {
    & $makensisPath /VERSION | Out-Null
    Write-Host "NSIS available: $makensisPath"
}

Write-Host "Checking Rust toolchain (from workflow)..."
rustc --version
cargo --version

# ----------------------------- #
# 1) vcpkg (leptonica + tesseract)  <-- binarysource REMOVED for compatibility
# ----------------------------- #
$userHome  = $env:USERPROFILE
$vcpkgRoot = Join-Path $userHome "vcpkg"
$vcpkgExe  = Join-Path $vcpkgRoot "vcpkg.exe"

if (-not (Test-Path $vcpkgExe)) {
    Write-Host "Installing vcpkg..."
    git clone --depth=1 https://github.com/microsoft/vcpkg $vcpkgRoot | Out-Null
    & (Join-Path $vcpkgRoot "bootstrap-vcpkg.bat") -disableMetrics | Out-Null
}

# Base env for vcpkg + vcpkg-rs
$env:VCPKG_ROOT                 = $vcpkgRoot
$env:VCPKG_DEFAULT_TRIPLET      = "x64-windows"
$env:VCPKGRS_TRIPLET            = "x64-windows"
$env:VCPKGRS_DYNAMIC            = "1"
$env:VCPKG_FEATURE_FLAGS        = "manifests,binarycaching"
$env:CMAKE_BUILD_PARALLEL_LEVEL = "1"
$env:VCPKG_MAX_CONCURRENCY      = "1"

# Ensure installed tree isn't a broken cached snapshot (delete if inconsistent)
$installedDir = Join-Path $vcpkgRoot "installed"
$metaDir      = Join-Path $installedDir "vcpkg"
$updatesDir   = Join-Path $metaDir "updates"
$statusFile   = Join-Path $metaDir "status"

$needReset = $false
if (Test-Path $installedDir) {
    if (-not (Test-Path $metaDir))     { $needReset = $true }
    if (-not (Test-Path $updatesDir))  { $needReset = $true }
    if (-not (Test-Path $statusFile))  { $needReset = $true }
}

if ($needReset) {
    Write-Warning "vcpkg 'installed' state looks inconsistent. Resetting installed tree..."
    Remove-Item $installedDir -Recurse -Force -ErrorAction SilentlyContinue
}

# Logging
$logDir = "C:\vcpkg-logs"
if (!(Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir | Out-Null }
$vcpkgLog = Join-Path $logDir "vcpkg_install.log"
$vcpkgList = Join-Path $logDir "vcpkg_list.log"

Write-Host "Installing vcpkg ports (leptonica, tesseract) — no binary cache (slower but compatible)..."

# Use explicit triplet appended to package names
$pkgs = @('leptonica:x64-windows','tesseract:x64-windows')

# Run and capture output (no binarysource)
& $vcpkgExe 'install' $pkgs[0] $pkgs[1] '--clean-after-build' '--triplet' 'x64-windows' '--debug' 2>&1 | Tee-Object -FilePath $vcpkgLog
if ($LASTEXITCODE -ne 0) {
    Write-Host "vcpkg install exited with code $LASTEXITCODE. See $vcpkgLog"
    throw "vcpkg install failed (exit $LASTEXITCODE)."
}

# capture list
& $vcpkgExe list --triplet x64-windows 2>&1 | Tee-Object -FilePath $vcpkgList

# Verify vcpkg installed ports for the requested triplet
$leptOk = Select-String -Path $vcpkgList -Pattern '^\s*leptonica:.*x64-windows' -Quiet
if (-not $leptOk) {
    Write-Host "vcpkg list output (first 200 lines):"
    Get-Content $vcpkgList | Select-Object -First 200 | ForEach-Object { Write-Host $_ }
    throw "vcpkg did not install leptonica:x64-windows (see $vcpkgLog)."
}
$tessOk = Select-String -Path $vcpkgList -Pattern '^\s*tesseract:.*x64-windows' -Quiet
if (-not $tessOk) {
    Write-Host "vcpkg list output (first 200 lines):"
    Get-Content $vcpkgList | Select-Object -First 200 | ForEach-Object { Write-Host $_ }
    throw "vcpkg did not install tesseract:x64-windows (see $vcpkgLog)."
}

Write-Host "vcpkg: leptonica & tesseract installed OK."

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
