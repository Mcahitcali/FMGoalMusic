<#
Robust Windows build script for FMGoalMusic (GitHub Actions)
- Bootstraps vcpkg (no fragile --binarysource)
- Installs leptonica and tesseract (x64-windows)
- Builds cargo release for x86_64-pc-windows-msvc
- Stages runtime files into build/windows/
- Writes vcpkg logs to C:\vcpkg-logs\vcpkg_install.log for artifact upload
Notes:
- This script intentionally does NOT call makensis. The workflow will call makensis so it can resolve its path reliably.
- Do NOT attempt to set $env:HOME here (runner HOME can be read-only).
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Log { param($m) Write-Host $m }

Write-Host "========== FM Goal Musics - Windows Setup & Build =========="

# --- Paths & basic artifacts
$scriptPath = $MyInvocation.MyCommand.Definition
$scriptDir  = Split-Path -Parent $scriptPath
$repoRoot   = Resolve-Path (Join-Path $scriptDir "..") | Select-Object -ExpandProperty Path

# vcpkg logs dir (workflow will upload this path as artifact)
$vcpkgLogDir = "C:\vcpkg-logs"
if (!(Test-Path $vcpkgLogDir)) { New-Item -ItemType Directory -Path $vcpkgLogDir | Out-Null }
$vcpkgLog = Join-Path $vcpkgLogDir "vcpkg_install.log"

# --- Helpers
function Fail($msg) {
    Write-Host "[ERROR] $msg" -ForegroundColor Red
    throw $msg
}

function Run-Checked($exe, $args) {
    Write-Host "Running: $exe $($args -join ' ')"
    & $exe @args 2>&1 | Tee-Object -Variable _runout
    if ($LASTEXITCODE -ne 0) {
        $outText = if ($null -ne $_runout) { ($_runout -join "`n") } else { "<no output>" }
        Fail "Command failed (exit $LASTEXITCODE): $exe $($args -join ' ')`n$outText"
    }
}

# --- Detect basic commands (warning, not fatal)
$tools = @("git","cmake","7z","pwsh")
foreach ($t in $tools) {
    if (Get-Command $t -ErrorAction SilentlyContinue) {
        Write-Host "$t: $((Get-Command $t).Path)"
    } else {
        Write-Host "Warning: $t not on PATH"
    }
}

# --- Import Visual Studio dev env robustly
# try common VsDevCmd locations
$vsCandidates = @(
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\Common7\Tools\VsDevCmd.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\Common7\Tools\VsDevCmd.bat"
)
$vsDevCmd = $vsCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $vsDevCmd) {
    Fail "VsDevCmd.bat not found. Ensure Visual Studio Build Tools are installed on runner."
}

Write-Host "Importing VS dev environment from: $vsDevCmd"

# create wrapper batch to call VsDevCmd and dump environment
$tempBat = Join-Path $env:TEMP "vsenv_wrapper.bat"
$tempEnvOut = Join-Path $env:TEMP "vsenv_out.txt"
$batContent = "@echo off`r`ncall `"$vsDevCmd`" -host_arch=amd64 -arch=amd64`r`nset > `"$tempEnvOut`""
Set-Content -Path $tempBat -Value $batContent -Encoding ASCII

# run wrapper and import env
$cmdOut = & cmd /c "`"$tempBat`"" 2>&1
if (-not (Test-Path $tempEnvOut)) {
    Write-Host "VsDevCmd run output:"
    Write-Host $cmdOut
    Fail "Failed to capture VS environment. Expected file: $tempEnvOut"
}

Get-Content $tempEnvOut | ForEach-Object {
    if ($_ -match '^(.*?)=(.*)$') {
        $name = $matches[1]; $value = $matches[2]
        if ($name -and ($name -notmatch '^[0-9]')) {
            # set in process env
            [System.Environment]::SetEnvironmentVariable($name, $value, 'Process')
        }
    }
}
Write-Host "MSVC environment imported."

# quick check for cl.exe
if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
    Fail "MSVC (cl.exe) not on PATH after VsDevCmd import."
}

# --- Ensure some tools via Chocolatey if missing (non-fatal to attempt)
function Ensure-ChocoInstalled {
    if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Host "Chocolatey not found on runner. This script assumes runner image has choco, or you installed VS/NSIS manually."
    }
}
Ensure-ChocoInstalled

# try to install nsis and tesseract if missing (best-effort)
if (-not (Get-Command makensis -ErrorAction SilentlyContinue)) {
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-Host "Installing NSIS via choco (best-effort)..."
        & choco install nsis.install -y --no-progress 2>&1 | Tee-Object -FilePath (Join-Path $vcpkgLogDir "choco_nsis.log")
        refreshenv | Out-Null
    } else {
        Write-Host "makensis not found and choco not available; makensis step will be run from workflow which will attempt to resolve makensis path."
    }
}

if (-not (Get-Command tesseract -ErrorAction SilentlyContinue)) {
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-Host "Installing Tesseract runtime via choco (best-effort)..."
        & choco install tesseract -y --no-progress 2>&1 | Tee-Object -FilePath (Join-Path $vcpkgLogDir "choco_tesseract.log")
        refreshenv | Out-Null
    } else {
        Write-Host "tesseract not found and choco not available."
    }
}

# --- vcpkg bootstrap & install (no binarysource) -----------------------------
$vcpkgRoot = Join-Path $env:USERPROFILE "vcpkg"
$vcpkgExe  = Join-Path $vcpkgRoot "vcpkg.exe"

if (-not (Test-Path $vcpkgExe)) {
    Write-Host "Cloning vcpkg..."
    if (Test-Path $vcpkgRoot) { Remove-Item -Recurse -Force $vcpkgRoot -ErrorAction SilentlyContinue }
    Run-Checked "git" @("clone","--depth","1","https://github.com/microsoft/vcpkg.git",$vcpkgRoot)
    Push-Location $vcpkgRoot
    Write-Host "Bootstrapping vcpkg..."
    & .\bootstrap-vcpkg.bat -disableMetrics 2>&1 | Tee-Object -FilePath $vcpkgLog
    if ($LASTEXITCODE -ne 0) { Fail "vcpkg bootstrap failed (exit $LASTEXITCODE). See $vcpkgLog" }
    Pop-Location
} else {
    Write-Host "vcpkg already present at $vcpkgRoot"
}

# set env used by vcpkg-aware build scripts
[System.Environment]::SetEnvironmentVariable("VCPKG_ROOT", $vcpkgRoot, "Process")
[System.Environment]::SetEnvironmentVariable("VCPKG_DEFAULT_TRIPLET", "x64-windows", "Process")
[System.Environment]::SetEnvironmentVariable("VCPKGRS_TRIPLET", "x64-windows", "Process")
[System.Environment]::SetEnvironmentVariable("VCPKGRS_DYNAMIC", "1", "Process")
[System.Environment]::SetEnvironmentVariable("VCPKG_FEATURE_FLAGS", "manifests,binarycaching", "Process")

# install ports (this may take long)
Write-Host "Installing vcpkg ports: leptonica, tesseract (triplet x64-windows)... (logs -> $vcpkgLog)"
Push-Location $vcpkgRoot
if (Test-Path $vcpkgLog) { Remove-Item $vcpkgLog -Force -ErrorAction SilentlyContinue }

# run vcpkg install and capture everything to vcpkg log
$installArgs = @("install","leptonica","tesseract","--triplet","x64-windows","--clean-after-build","-v")
$proc = Start-Process -FilePath $vcpkgExe -ArgumentList $installArgs -NoNewWindow -RedirectStandardOutput $vcpkgLog -RedirectStandardError $vcpkgLog -PassThru -Wait
if ($proc.ExitCode -ne 0) {
    Get-Content $vcpkgLog -Tail 200 | ForEach-Object { Write-Host $_ }
    Fail "vcpkg install failed (exit $($proc.ExitCode)). See $vcpkgLog"
}

# verify list
$listOut = & $vcpkgExe list --triplet x64-windows 2>&1
Write-Host "vcpkg list (tail):"
$listOut | Select-Object -Last 40 | ForEach-Object { Write-Host $_ }

if ($listOut -notmatch "leptonica:.*x64-windows") { Fail "leptonica not found in vcpkg list; see $vcpkgLog" }
if ($listOut -notmatch "tesseract:.*x64-windows")  { Fail "tesseract not found in vcpkg list; see $vcpkgLog" }

Pop-Location
Write-Host "vcpkg: leptonica & tesseract installed."

# --- Build Rust release -----------------------------------------------------
Write-Host "[1/2] Building Rust -- release (MSVC target)"
Push-Location $repoRoot
# cargo should be available via runner tool setup; preserve RUSTFLAGS if set
$env:RUSTFLAGS = $env:RUSTFLAGS
Run-Checked "cargo" @("build","--release","--target","x86_64-pc-windows-msvc") 
Pop-Location

# verify artifact
$exeName = "fm-goal-musics-gui.exe"
$exeRel  = Join-Path $repoRoot "target\x86_64-pc-windows-msvc\release\$exeName"
if (-not (Test-Path $exeRel)) { Fail "Built binary not found: $exeRel" }

# --- Stage runtime: create build/windows and copy files ---------------------
$buildDir = Join-Path $repoRoot "build\windows"
if (Test-Path $buildDir) { Remove-Item -Recurse -Force $buildDir -ErrorAction SilentlyContinue }
New-Item -ItemType Directory -Path $buildDir | Out-Null

Copy-Item $exeRel -Destination $buildDir -Force
Write-Host "Copied binary to $buildDir"

# include config/assets if present
$maybeAssets = @("config","assets","README.md","LICENSE")
foreach ($it in $maybeAssets) {
    $src = Join-Path $repoRoot $it
    if (Test-Path $src) {
        Copy-Item $src -Destination $buildDir -Recurse -Force
        Write-Host "Included: $it"
    }
}

# copy vcpkg runtime DLLs (if available)
$vcpkgBin = Join-Path $vcpkgRoot "installed\x64-windows\bin"
if (Test-Path $vcpkgBin) {
    Copy-Item (Join-Path $vcpkgBin "*.dll") -Destination $buildDir -Force -ErrorAction SilentlyContinue
    Write-Host "Included vcpkg DLLs from $vcpkgBin"
} else {
    Write-Host "Warning: vcpkg bin folder not found; runtime DLLs may be missing."
}

# copy tessdata if present
$tessdataSrc = Join-Path $vcpkgRoot "installed\x64-windows\share\tesseract\tessdata"
if (Test-Path $tessdataSrc) {
    Copy-Item $tessdataSrc -Destination (Join-Path $buildDir "tessdata") -Recurse -Force
    Write-Host "Included tessdata"
} else {
    Write-Host "Warning: tessdata not found in vcpkg installed tree."
}

Write-Host "[2/2] Build staging complete — payload is in: $buildDir"
Write-Host "Done."
