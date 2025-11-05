# build_for_windows.ps1
# Robust Windows build + stage script for your self-hosted runner.
# Designed to match the Release workflow (uses VCPKG_ROOT and LIBCLANG_PATH if provided).

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Write-Host "=== build_for_windows.ps1 starting ==="

# 1) Environment defaults (can be overridden by workflow env)
if (-not $env:VCPKG_ROOT -or $env:VCPKG_ROOT -eq '') {
    $env:VCPKG_ROOT = 'C:\vcpkg'
    Write-Host "VCPKG_ROOT not set; defaulting to $env:VCPKG_ROOT"
} else {
    Write-Host "VCPKG_ROOT = $env:VCPKG_ROOT"
}

if (-not $env:LIBCLANG_PATH -or $env:LIBCLANG_PATH -eq '') {
    $env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
    Write-Host "LIBCLANG_PATH not set; defaulting to $env:LIBCLANG_PATH"
} else {
    Write-Host "LIBCLANG_PATH = $env:LIBCLANG_PATH"
}

# Ensure PATH contains LIBCLANG_PATH and Git bash if present
if (Test-Path $env:LIBCLANG_PATH) {
    if ($env:Path -notlike "*$env:LIBCLANG_PATH*") {
        Write-Host "Adding LIBCLANG_PATH to PATH for this process"
        $env:Path = "$env:LIBCLANG_PATH;$env:Path"
    }
} else {
    Write-Warning "LIBCLANG_PATH path does not exist: $env:LIBCLANG_PATH"
}

# Add common Git-for-Windows locations to PATH for steps that expect bash
$gitPaths = @(
    'C:\Program Files\Git\usr\bin',
    'C:\Program Files\Git\bin',
    'C:\Program Files (x86)\Git\usr\bin',
    'C:\Program Files (x86)\Git\bin'
)
foreach ($p in $gitPaths) {
    if (Test-Path $p -and $env:Path -notlike "*$p*") {
        Write-Host "Adding $p to PATH"
        $env:Path = "$p;$env:Path"
        break
    }
}

# 2) Useful debug info (helps when runner fails)
Write-Host "---- Env summary ----"
Write-Host "CARGO_BUILD_JOBS = $env:CARGO_BUILD_JOBS"
Write-Host "RUSTFLAGS = $env:RUSTFLAGS"
Write-Host "RUST_BACKTRACE = $env:RUST_BACKTRACE"
Write-Host "PATH contains first 6 entries:"
$env:Path.Split(';')[0..6] -join "`n" | Write-Host

if (Test-Path (Join-Path $env:VCPKG_ROOT 'vcpkg.exe')) {
    Write-Host "vcpkg found:"
    & "$($env:VCPKG_ROOT)\vcpkg.exe" list | Select-String -Pattern 'leptonica|tesseract|tiff' -Quiet | Out-Null
} else {
    Write-Warning "vcpkg.exe not found under $env:VCPKG_ROOT. If you rely on vcpkg, install or set VCPKG_ROOT correctly."
}

# 3) Choose vcpkg triplet to copy runtime DLLs from
$tripletCandidates = @('x64-windows', 'x64-windows-static', 'x64-windows-static-md', 'x64-windows-md')
$selectedTriplet = $null
foreach ($t in $tripletCandidates) {
    if (Test-Path (Join-Path $env:VCPKG_ROOT "installed\$t")) {
        $selectedTriplet = $t
        break
    }
}
if (-not $selectedTriplet) {
    Write-Warning "No known vcpkg triplet folder found under $env:VCPKG_ROOT\installed. Falling back to 'x64-windows' if present."
    if (Test-Path (Join-Path $env:VCPKG_ROOT 'installed\x64-windows')) { $selectedTriplet = 'x64-windows' }
}
Write-Host "Selected vcpkg triplet: $selectedTriplet"

# 4) Clean old staged files (optional)
$repoRoot = (Get-Location).ProviderPath
$buildWindows = Join-Path $repoRoot 'build\windows'
if (Test-Path $buildWindows) {
    Write-Host "Cleaning existing build/windows contents..."
    Get-ChildItem $buildWindows -Recurse -Force | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $buildWindows | Out-Null

# 5) Run cargo build --release
Write-Host "Starting cargo build --release"
# Ensure single-job builds on low-memory runners if provided by workflow env
$cargoArgs = @('build','--release')
# If you want locked builds uncomment next line:
# $cargoArgs += '--locked'
$start = Get-Date
try {
    & cargo @cargoArgs
} catch {
    Write-Error "cargo build failed: $($_.Exception.Message)"
    # print a bit more debug
    Write-Host "=== cargo env ==="
    cargo --version | Write-Host
    Write-Host "Printing last 200 lines of cargo build log (if any)..."
    throw
}
$duration = (Get-Date) - $start
Write-Host "cargo build completed in $($duration.TotalMinutes) minutes"

# 6) Find main exe in target\release (exclude build-script exes)
$targetRelease = Join-Path $repoRoot 'target\release'
$exeCandidates = Get-ChildItem -Path $targetRelease -Filter *.exe -File -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -notmatch '\\build\\' } |
    Sort-Object Length -Descending

if ($exeCandidates.Count -eq 0) {
    Write-Error "No exe found in $targetRelease (after build). Aborting."
    exit 1
}
# pick largest non-deps exe
$mainExe = $exeCandidates | Select-Object -First 1
Write-Host "Selected exe: $($mainExe.FullName) ($([Math]::Round($mainExe.Length/1MB,2)) MB)"

# 7) Copy exe to build/windows
Copy-Item -Path $mainExe.FullName -Destination (Join-Path $buildWindows $mainExe.Name) -Force
Write-Host "Copied exe -> $buildWindows\$($mainExe.Name)"

# 8) Copy any DLLs from target\release (some dlls appear there)
Get-ChildItem -Path $targetRelease -Filter *.dll -File -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "Copying target dll: $($_.Name)"
    Copy-Item $_.FullName -Destination $buildWindows -Force
}

# 9) Copy vcpkg-provided runtime DLLs (from selected triplet)
if ($selectedTriplet) {
    $vcpkgBin = Join-Path $env:VCPKG_ROOT "installed\$selectedTriplet\bin"
    $vcpkgLib = Join-Path $env:VCPKG_ROOT "installed\$selectedTriplet\lib"
    $pathsToCopy = @()
    if (Test-Path $vcpkgBin) { $pathsToCopy += $vcpkgBin }
    if (Test-Path $vcpkgLib) { $pathsToCopy += $vcpkgLib }
    foreach ($p in $pathsToCopy) {
        Get-ChildItem -Path $p -Filter *.dll -File -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host "Copying vcpkg dll: $($_.Name) from $p"
            Copy-Item $_.FullName -Destination $buildWindows -Force
        }
    }

    # Also copy large runtime files like zlib1.dll etc from installed\$triplet\debug or 'tools' if present
    $maybePaths = @(
        Join-Path $env:VCPKG_ROOT "installed\$selectedTriplet\debug\bin",
        Join-Path $env:VCPKG_ROOT "installed\$selectedTriplet\tools"
    )
    foreach ($mp in $maybePaths) {
        if (Test-Path $mp) {
            Get-ChildItem -Path $mp -Filter *.dll -File -ErrorAction SilentlyContinue | ForEach-Object {
                Copy-Item $_.FullName -Destination $buildWindows -Force
            }
        }
    }
} else {
    Write-Warning "No vcpkg triplet selected; skipping vcpkg dll copy step."
}

# 10) Copy repository-provided assets that NSIS expects (if exist)
$repoAssets = @('app.ico','icon.icns','readme.md') # adjust if you have other files
foreach ($f in $repoAssets) {
    $src = Join-Path $repoRoot $f
    if (Test-Path $src) {
        Copy-Item $src -Destination $buildWindows -Force
        Write-Host "Copied asset: $f"
    }
}

# 11) Post-check: list final staged files
Write-Host "=== Staged build/windows contents ==="
Get-ChildItem -Path $buildWindows -Recurse | Sort-Object Length -Descending | Select-Object Name,Length | Format-Table -AutoSize

Write-Host "=== build_for_windows.ps1 finished successfully ==="
exit 0
