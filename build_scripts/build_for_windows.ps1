# build_for_windows.ps1
# Updated to match release.yml expectations:
# - honors VCPKG_ROOT, LIBCLANG_PATH, CARGO_BUILD_JOBS, RUSTFLAGS, RUST_BACKTRACE
# - builds release, stages binary into build/windows
# - copies NSIS temporary output if present
# - fails loudly on errors

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Write-Host "=== build_for_windows.ps1 starting ==="
Write-Host "PWD: $(Get-Location)"

# --- Ensure env vars (workflow sets these; use sensible defaults if missing) ---
if (-not $env:VCPKG_ROOT) {
    $env:VCPKG_ROOT = "C:\vcpkg"
    Write-Host "VCPKG_ROOT not set, defaulting to $env:VCPKG_ROOT"
} else { Write-Host "VCPKG_ROOT = $env:VCPKG_ROOT" }

if (-not $env:LIBCLANG_PATH) {
    $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
    Write-Host "LIBCLANG_PATH not set, defaulting to $env:LIBCLANG_PATH"
} else { Write-Host "LIBCLANG_PATH = $env:LIBCLANG_PATH" }

if (-not $env:CARGO_BUILD_JOBS) { $env:CARGO_BUILD_JOBS = "1" }
if (-not $env:RUSTFLAGS)        { $env:RUSTFLAGS = "-C codegen-units=1" }
if (-not $env:RUST_BACKTRACE)   { $env:RUST_BACKTRACE = "1" }

Write-Host "CARGO_BUILD_JOBS = $env:CARGO_BUILD_JOBS"
Write-Host "RUSTFLAGS = $env:RUSTFLAGS"
Write-Host "RUST_BACKTRACE = $env:RUST_BACKTRACE"

# Print quick versions (helpful for debugging on runner)
try {
    & rustc --version
    & cargo --version
} catch {
    Write-Warning "rustc/cargo not found in PATH. Ensure the 'Install Rust' step of workflow ran successfully."
}

# --- Clean previous build artifacts (avoid stale outputs) ---
if (Test-Path ".\target") {
    Write-Host "Removing ./target directory..."
    Remove-Item -Recurse -Force .\target
} else {
    Write-Host "No ./target directory to remove."
}

# --- vcpkg debug info (if present) ---
if (Test-Path (Join-Path $env:VCPKG_ROOT "vcpkg.exe")) {
    Write-Host "vcpkg detected at $env:VCPKG_ROOT\vcpkg.exe"
    Write-Host "Listing installed vcpkg packages that matter (leptonica/tesseract/tiff)..."
    & "$env:VCPKG_ROOT\vcpkg.exe" list | Select-String -Pattern 'leptonica|tesseract|tiff' -Quiet:$false
} else {
    Write-Warning "vcpkg not found at $env:VCPKG_ROOT\vcpkg.exe — native deps may fail if not installed."
}

# Ensure LIBCLANG_PATH contains libclang.dll (bindgen requirement)
if (-not (Test-Path (Join-Path $env:LIBCLANG_PATH 'libclang.dll'))) {
    Write-Warning "libclang.dll not found at $env:LIBCLANG_PATH. bindgen may fail. Make sure LLVM is installed and LIBCLANG_PATH is correct."
} else {
    Write-Host "libclang.dll present at $env:LIBCLANG_PATH"
}

# --- Run cargo build (release) ---
Write-Host "Running cargo build --release (single job: $env:CARGO_BUILD_JOBS)..."
# Save output to files for debugging in case of failure.
$stdoutFile = "build_scripts\cargo.build.out.txt"
$stderrFile = "build_scripts\cargo.build.err.txt"
# Ensure build_scripts dir exists
if (-not (Test-Path "build_scripts")) { New-Item -ItemType Directory -Path "build_scripts" -Force | Out-Null }

# Start build (environment variables already set above)
$buildArgs = @("build","--release")
$proc = Start-Process -FilePath "cargo" -ArgumentList $buildArgs -NoNewWindow -Wait -PassThru `
    -RedirectStandardOutput $stdoutFile -RedirectStandardError $stderrFile

if ($proc.ExitCode -ne 0) {
    Write-Host "=== cargo build FAILED ==="
    Write-Host "---- stdout (tail 200) ----"
    Get-Content $stdoutFile -Tail 200 | ForEach-Object { Write-Host $_ }
    Write-Host "---- stderr (tail 200) ----"
    Get-Content $stderrFile -Tail 200 | ForEach-Object { Write-Host $_ }
    throw "cargo build --release failed with exit code $($proc.ExitCode)"
}
Write-Host "cargo build succeeded."

# --- Stage artifacts into build/windows ---
$stagingDir = "build\windows"
if (-not (Test-Path $stagingDir)) {
    New-Item -ItemType Directory -Path $stagingDir -Force | Out-Null
    Write-Host "Created staging dir $stagingDir"
}

# Prefer direct target/release exe (largest exe)
Write-Host "Searching for produced exe in target\release ..."
$exeCandidates = Get-ChildItem -Path "target\release" -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue |
                 Where-Object { $_.FullName -notmatch "\\build\\" } |
                 Sort-Object Length -Descending

if ($exeCandidates -and $exeCandidates.Count -gt 0) {
    $chosen = $exeCandidates[0]
    Write-Host "Choosing built exe: $($chosen.FullName)"
    Copy-Item -Path $chosen.FullName -Destination (Join-Path $stagingDir $chosen.Name) -Force
} else {
    Write-Host "No exe found directly; searching target\release\deps ..."
    $depExe = Get-ChildItem -Path "target\release\deps" -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue |
              Sort-Object Length -Descending | Select-Object -First 1
    if ($depExe) {
        Write-Host "Using deps exe: $($depExe.FullName)"
        Copy-Item -Path $depExe.FullName -Destination (Join-Path $stagingDir $depExe.Name) -Force
    } else {
        Write-Error "No built executable found to stage. Aborting."
        throw "No built exe to stage."
    }
}

# If NSIS in some runs created installer under build_scripts\build\windows, copy it to standard staging
$nsisTempInstaller = "build_scripts\build\windows\FMGoalMusicInstaller.exe"
if (Test-Path $nsisTempInstaller) {
    Write-Host "Found NSIS temporary installer at $nsisTempInstaller - copying to $stagingDir"
    Copy-Item -Path $nsisTempInstaller -Destination (Join-Path $stagingDir "FMGoalMusicInstaller.exe") -Force
}

# Also keep any repo-provided helper files already in build/windows (icons, dlls). Show staged files:
Write-Host "Staged files in $stagingDir:"
Get-ChildItem $stagingDir | Select-Object Name, Length | Format-Table -AutoSize

Write-Host "=== build_for_windows.ps1 finished successfully ==="
