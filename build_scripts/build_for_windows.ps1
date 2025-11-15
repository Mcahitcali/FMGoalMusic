# build_scripts/build_for_windows.ps1
# Robust build script for GitHub self-hosted Windows runner (pwsh).
# Avoids PowerShell parsing pitfalls by using Start-Process for external commands,
# explicit path quoting and clear error handling.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host "=== build_for_windows.ps1 starting ==="

# --- ENV defaults (mirror release.yml) ---
$autoDetectedJobs = $false
if (-not $env:CARGO_BUILD_JOBS -or $env:CARGO_BUILD_JOBS -eq '') {
    $env:CARGO_BUILD_JOBS = [Environment]::ProcessorCount.ToString()
    $autoDetectedJobs = $true
}
if (-not $env:RUST_BACKTRACE -or $env:RUST_BACKTRACE -eq '') { $env:RUST_BACKTRACE = '1' }
if (-not $env:VCPKG_ROOT -or $env:VCPKG_ROOT -eq '') { $env:VCPKG_ROOT = 'C:\vcpkg' }
if (-not $env:LIBCLANG_PATH -or $env:LIBCLANG_PATH -eq '') { $env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin' }
if (-not $env:VCPKG_DEFAULT_TRIPLET -or $env:VCPKG_DEFAULT_TRIPLET -eq '') { $env:VCPKG_DEFAULT_TRIPLET = 'x64-windows-static' }

Write-Host "Env summary:"
if ($autoDetectedJobs) {
    Write-Host "  CARGO_BUILD_JOBS    = $($env:CARGO_BUILD_JOBS) (auto-detected from CPU cores)"
} else {
    Write-Host "  CARGO_BUILD_JOBS    = $($env:CARGO_BUILD_JOBS)"
}
Write-Host "  RUSTFLAGS           = $($env:RUSTFLAGS)"
Write-Host "  RUST_BACKTRACE      = $($env:RUST_BACKTRACE)"
Write-Host "  VCPKG_ROOT          = $($env:VCPKG_ROOT)"
Write-Host "  VCPKG_DEFAULT_TRIPLET = $($env:VCPKG_DEFAULT_TRIPLET)"
Write-Host "  LIBCLANG_PATH       = $($env:LIBCLANG_PATH)"
Write-Host ""

# --- Ensure common tools are visible in PATH ---
$possiblePaths = @(
    'C:\Program Files\Git\usr\bin',
    'C:\Program Files\Git\bin',
    'C:\Program Files (x86)\Git\usr\bin',
    'C:\Program Files (x86)\Git\bin',
    'C:\Program Files (x86)\NSIS',
    'C:\Program Files\LLVM\bin'
)
foreach ($p in $possiblePaths) {
    if (Test-Path $p) {
        if ($env:Path -notlike "*$p*") {
            Write-Host "Adding $p to PATH"
            $env:Path = "$env:Path;$p"
        }
    }
}

# --- vcpkg checks ---
$vcpkgExe = Join-Path $env:VCPKG_ROOT 'vcpkg.exe'
if (-not (Test-Path $vcpkgExe)) {
    Write-Error "vcpkg.exe not found at $vcpkgExe. Install vcpkg on runner or adjust VCPKG_ROOT."
    exit 1
}
Write-Host "`nvcpkg present at: $vcpkgExe"

$triplet = $env:VCPKG_DEFAULT_TRIPLET
# ✅ FIXED LINE: use $() to safely interpolate variable with colon
$packages = @('leptonica','tesseract','zlib','libpng','libtiff','libjpeg-turbo') | ForEach-Object { "$($_):$triplet" }

# capture vcpkg list once (refresh if installs occur)
$vcpkgListOutput = (& $vcpkgExe list)

# print vcpkg list (filtered)
Write-Host "`n== vcpkg list (filtered) =="
$vcpkgListOutput | Select-String -Pattern 'leptonica|tesseract|zlib|libpng|libtiff|jpeg' -SimpleMatch | ForEach-Object { Write-Host $_.ToString() }

# Attempt install if any package missing (best-effort)
foreach ($pkg in $packages) {
    Write-Host "`nChecking $pkg ..."
    $pkgCheck = [regex]::Escape(($pkg.Split(':')[0] + ':'))
    if ((($vcpkgListOutput -join "`n") -notmatch $pkgCheck)) {
        Write-Host "$pkg not clearly present in vcpkg list — attempting install (may be slow)..."
        $rc = Start-Process -FilePath $vcpkgExe -ArgumentList @('install', $pkg) -NoNewWindow -Wait -PassThru
        if ($rc.ExitCode -ne 0) {
            Write-Warning "vcpkg install $pkg failed with exit code $($rc.ExitCode). Continue but cargo may fail later."
        } else {
            Write-Host "Installed $pkg via vcpkg."
            $vcpkgListOutput = (& $vcpkgExe list)
        }
    } else {
        Write-Host "$pkg already appears installed (skipping)."
    }
}

# integrate (best-effort)
try {
    $rc = Start-Process -FilePath $vcpkgExe -ArgumentList @('integrate','install') -NoNewWindow -Wait -PassThru
    if ($rc.ExitCode -eq 0) { Write-Host "vcpkg integrate ok." } else { Write-Host "vcpkg integrate returned $($rc.ExitCode)." }
} catch {
    Write-Host "vcpkg integrate failed or already integrated: $_"
}

# --- Clean target (optional) ---
$cleanRequested = $false
if ($env:FMGOALMUSIC_CLEAN_BUILD) {
    $cleanRequested = $env:FMGOALMUSIC_CLEAN_BUILD.ToLower() -in @('1','true','yes')
}
if ($cleanRequested) {
    Write-Host "`nCleaning previous build artifacts (FMGOALMUSIC_CLEAN_BUILD enabled)..."
    if (Test-Path '.\target') {
        Remove-Item -Recurse -Force .\target
        Write-Host "Removed ./target"
    } else {
        Write-Host "No ./target to remove"
    }
} else {
    Write-Host "`nSkipping cargo clean to reuse incremental build cache. Set FMGOALMUSIC_CLEAN_BUILD=1 to force clean."
}

# --- Cargo build (release) ---
Write-Host "`nStarting cargo build --release ..."
$cargo = 'cargo'
$cargoArgs = @('build','--release')
if ($env:CARGO_BUILD_JOBS -and $env:CARGO_BUILD_JOBS -ne '') {
    $cargoArgs += @('--jobs',$env:CARGO_BUILD_JOBS)
}
$proc = Start-Process -FilePath $cargo -ArgumentList $cargoArgs -NoNewWindow -Wait -PassThru
if ($proc.ExitCode -ne 0) {
    Write-Error "cargo build failed with exit code $($proc.ExitCode)"
    exit $proc.ExitCode
}
Write-Host "cargo build succeeded."

# --- Stage artifacts into build/windows ---
$repoRoot = Get-Location
$stageDir = Join-Path $repoRoot 'build\windows'
if (-not (Test-Path $stageDir)) { New-Item -ItemType Directory -Path $stageDir | Out-Null }

# find the largest exe in target/release
$exeFiles = Get-ChildItem -Path (Join-Path $repoRoot 'target\release') -Filter *.exe -Recurse -ErrorAction SilentlyContinue |
           Where-Object { $_.FullName -notmatch '\\build\\' } |
           Sort-Object Length -Descending
if ($exeFiles.Count -eq 0) {
    Write-Error "No exe found in target/release (build failed or different binary name)."
    exit 1
}
$mainExe = $exeFiles[0].FullName
Write-Host "Selected exe to stage: $mainExe"
Copy-Item -Path $mainExe -Destination $stageDir -Force
Write-Host "Copied exe -> $stageDir"

# Copy vcpkg runtime DLLs
$vcpkgBin = Join-Path $env:VCPKG_ROOT "installed\$triplet\bin"
$vcpkgLib = Join-Path $env:VCPKG_ROOT "installed\$triplet\lib"
if (Test-Path $vcpkgBin) {
    Get-ChildItem -Path $vcpkgBin -Filter *.dll -File -ErrorAction SilentlyContinue | ForEach-Object {
        Copy-Item $_.FullName -Destination $stageDir -Force
    }
    Write-Host "Copied DLLs from $vcpkgBin"
} elseif (Test-Path $vcpkgLib) {
    Get-ChildItem -Path $vcpkgLib -Filter *.dll -File -ErrorAction SilentlyContinue | ForEach-Object {
        Copy-Item $_.FullName -Destination $stageDir -Force
    }
    Write-Host "Copied DLLs from $vcpkgLib"
} else {
    Write-Warning "No vcpkg bin/lib directory found for triplet $triplet."
}

# Copy tessdata directory for bundled Tesseract data
$tessdataSrc = Join-Path $repoRoot 'tessdata'
if (Test-Path $tessdataSrc) {
    $tessdataDest = Join-Path $stageDir 'tessdata'
    Copy-Item -Path $tessdataSrc -Destination $tessdataDest -Recurse -Force
    Write-Host "Copied tessdata -> $tessdataDest"
} else {
    Write-Warning "tessdata directory not found at $tessdataSrc; OCR may require system Tesseract."
}

foreach ($res in @('app.ico','icon.icns')) {
    $src = Join-Path $repoRoot "build\windows\$res"
    if (Test-Path $src) { Copy-Item $src -Destination $stageDir -Force }
}

Write-Host "`nStaging complete. Artifacts are in: $stageDir"
Write-Host "=== build_for_windows.ps1 finished successfully ==="
exit 0
