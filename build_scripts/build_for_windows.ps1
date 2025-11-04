<#
build_for_windows.ps1
Robust Windows build for GitHub Actions (x86_64-pc-windows-msvc target)
- No emojis
- Handles VS dev environment import correctly
- Bootstraps vcpkg, integrates it, installs leptonica+tesseract for x64-windows
- Builds with cargo for MSVC target
- Creates NSIS installer
- Produces logs for vcpkg and NSIS
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Log {
    param([string]$msg)
    Write-Host $msg
}

# --- Basic paths
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path "$scriptDir\.." | Select-Object -ExpandProperty Path
$vcpkgRoot = Join-Path $env:USERPROFILE "vcpkg"   # will use this by default
$tempEnvFile = Join-Path $env:TEMP "vsdevcmd_env.txt"
$vcpkgLog = Join-Path $scriptDir "vcpkg_install.log"
$nsisLog = Join-Path $scriptDir "nsis.log"

# --- Helpers
function Run-Proc {
    param([string]$exe, [string[]]$args)
    $cmdline = @($exe) + $args -join ' '
    Write-Log ">>> Running: $cmdline"
    $p = Start-Process -FilePath $exe -ArgumentList $args -NoNewWindow -PassThru -Wait -RedirectStandardOutput ([IO.File]::CreateText("$scriptDir\proc_stdout.log")) -RedirectStandardError ([IO.File]::CreateText("$scriptDir\proc_stderr.log"))
    return $p.ExitCode
}

function Ensure-Choco {
    if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
        Write-Log "Chocolatey not found — installing minimal bootstrap..."
        Set-ExecutionPolicy Bypass -Scope Process -Force
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    } else {
        Write-Log "Chocolatey present"
    }
}

function Ensure-Tool-Choco {
    param([string]$name, [string]$chocoPkg)
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        Write-Log "$name not found on PATH; installing choco package: $chocoPkg"
        choco install $chocoPkg -y --no-progress
        Write-Log "Refreshing environment variables..."
        refreshenv | Out-Null
    } else {
        Write-Log "$name found: $(Get-Command $name)."
    }
}

# --- Import Visual Studio Developer Environment (robust)
function Import-VsDevEnv {
    Write-Log "Importing VS developer environment..."
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) {
        Write-Log "vswhere not found at expected path: $vswhere"
        throw "vswhere not found; cannot find Visual Studio installation."
    }

    $installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if (-not $installationPath) {
        # fallback to searching typical locations
        $possible = @(
            "C:\Program Files\Microsoft Visual Studio\2022\Enterprise",
            "C:\Program Files\Microsoft Visual Studio\2022\BuildTools",
            "C:\Program Files\Microsoft Visual Studio\2022\Professional",
            "C:\Program Files\Microsoft Visual Studio\2022\Community"
        ) | Where-Object { Test-Path $_ } | Select-Object -First 1
        $installationPath = $possible
    }

    if (-not $installationPath) {
        Write-Log "Visual Studio installation not found on runner. Ensure MSVC toolchain is present."
        throw "Visual Studio (MSVC) not found."
    }

    $vsDevCmd = Join-Path $installationPath "Common7\Tools\VsDevCmd.bat"
    if (-not (Test-Path $vsDevCmd)) {
        Write-Log "VsDevCmd.bat not found at: $vsDevCmd"
        throw "VsDevCmd not found"
    }

    # Run VsDevCmd.bat in cmd and then dump environment using 'set'
    Write-Log "Running VsDevCmd.bat and capturing environment..."
    $envLines = & cmd /c "`"$vsDevCmd`" -arch=amd64 -host_arch=amd64 1>nul 2>nul && set"
    if (-not $envLines) {
        throw "Failed to run VsDevCmd.bat or capture environment."
    }

    foreach ($line in $envLines) {
        if ($line -match "^[^=]+=.*$") {
            $parts = $line.Split("=",2)
            $key = $parts[0]
            $value = $parts[1]
            # Import into current process (Process scope)
            [System.Environment]::SetEnvironmentVariable($key, $value, 'Process')
        }
    }

    # quick verify cl.exe is reachable
    if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
        Write-Log "MSVC cl.exe is not on PATH after importing VS dev environment."
        throw "MSVC (cl/link) not on PATH"
    }
    Write-Log "MSVC imported successfully."
}

# --- Bootstrap vcpkg & install ports
function Ensure-Vcpkg {
    param(
        [string]$vcpkgRootParam = $vcpkgRoot,
        [string]$triplet = "x64-windows"
    )
    $vcpkgRootParam = Resolve-Path $vcpkgRootParam -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Path -ErrorAction SilentlyContinue
    if (-not $vcpkgRootParam) {
        $vcpkgRootParam = $vcpkgRoot
    }
    if (-not (Test-Path $vcpkgRootParam)) {
        Write-Log "Cloning vcpkg to $vcpkgRootParam..."
        git clone --depth 1 https://github.com/microsoft/vcpkg.git $vcpkgRootParam
    } else {
        Write-Log "vcpkg root exists at $vcpkgRootParam"
    }

    $vcpkgExe = Join-Path $vcpkgRootParam "vcpkg.exe"
    if (-not (Test-Path $vcpkgExe)) {
        Write-Log "Bootstrapping vcpkg..."
        & "$vcpkgRootParam\bootstrap-vcpkg.bat" | Tee-Object -FilePath $vcpkgLog
    } else {
        Write-Log "vcpkg.exe already present"
    }

    # Ensure VCPKG_ROOT env var is set for cargo build scripts that look for it
    [System.Environment]::SetEnvironmentVariable("VCPKG_ROOT", $vcpkgRootParam, 'Process')
    [System.Environment]::SetEnvironmentVariable("VCPKG_DEFAULT_TRIPLET", $triplet, 'Process')

    # Integrate vcpkg for MSBuild/CMake integration (makes find_package etc work)
    & $vcpkgExe integrate install | Tee-Object -FilePath $vcpkgLog

    # Install required ports (do NOT use broken custom binarysource flags)
    Write-Log "Installing vcpkg ports: leptonica, tesseract ($triplet)... (this may take a while)"
    $exit = & $vcpkgExe install leptonica tesseract --triplet $triplet --clean-after-build 2>&1 | Tee-Object -FilePath $vcpkgLog
    if ($LASTEXITCODE -ne 0) {
        Write-Log "vcpkg install output saved to $vcpkgLog"
        throw "vcpkg install failed (exit $LASTEXITCODE)."
    }

    # sanity checks: ensure packages appear in vcpkg list for the triplet
    $listOut = & $vcpkgExe list --triplet $triplet
    if ($listOut -notmatch "leptonica:.*$triplet") {
        throw "vcpkg did not install leptonica:$triplet (list not containing it). See $vcpkgLog"
    }
    if ($listOut -notmatch "tesseract:.*$triplet") {
        throw "vcpkg did not install tesseract:$triplet (list not containing it). See $vcpkgLog"
    }

    Write-Log "vcpkg ports installed successfully."
}

# --- Build the Rust project
function Build-Rust {
    param([string]$target="x86_64-pc-windows-msvc")
    Write-Log "Checking Rust toolchain..."
    $rustc = & rustc --version
    $cargo = & cargo --version
    Write-Log $rustc
    Write-Log $cargo

    Write-Log "Building cargo --release for target $target..."
    Push-Location $repoRoot
    $env:RUSTFLAGS = $env:RUSTFLAGS  # preserve if set
    $buildExit = & cargo build --release --target $target 2>&1 | Tee-Object -FilePath (Join-Path $scriptDir "cargo_build.log")
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Cargo build failed — see cargo_build.log"
        Pop-Location
        throw "Cargo build failed (exit $LASTEXITCODE)"
    }
    Pop-Location
    Write-Log "Cargo build completed."
}

# --- Make NSIS installer
function Create-Installer {
    param([string]$nsiPath = (Join-Path $scriptDir "FMGoalMusicInstaller.nsi"))
    Write-Log "Checking NSIS (makensis)..."
    Ensure-Tool-Choco -name "makensis" -chocoPkg "nsis"
    $makensisCmd = Get-Command makensis -ErrorAction SilentlyContinue
    if (-not $makensisCmd) {
        throw "makensis not found even after choco install. Check PATH."
    }
    Write-Log "Running makensis $nsiPath ..."
    & makensis /V4 $nsiPath 2>&1 | Tee-Object -FilePath $nsisLog
    if ($LASTEXITCODE -ne 0) {
        Write-Log "makensis failed; nsis.log saved at $nsisLog"
        throw "makensis failed with code $LASTEXITCODE"
    }
    Write-Log "NSIS installer created successfully."
}

# --- Script main
Write-Log "========== FM Goal Musics - Windows Setup & Build =========="

try {
    # Don't overwrite read-only HOME; use a safe local name if needed
    $userProfile = $env:USERPROFILE

    Ensure-Choco

    # Ensure helpful tools
    Ensure-Tool-Choco -name "cmake" -chocoPkg "cmake"
    Ensure-Tool-Choco -name "git" -chocoPkg "git"
    Ensure-Tool-Choco -name "7z" -chocoPkg "7zip"
    Ensure-Tool-Choco -name "pwsh" -chocoPkg "powershell-core"   # attempt to ensure pwsh available if needed

    # Import Visual Studio dev env
    Import-VsDevEnv

    # Ensure NSIS (makensis) and Tesseract (exe) for runtime if you want
    Ensure-Tool-Choco -name "makensis" -chocoPkg "nsis"
    Ensure-Tool-Choco -name "tesseract" -chocoPkg "tesseract"

    # Bootstrap and install vcpkg ports
    Ensure-Vcpkg -vcpkgRootParam $vcpkgRoot -triplet "x64-windows"

    # Build Rust
    Build-Rust -target "x86_64-pc-windows-msvc"

    # Create NSIS installer
    $nsi = Join-Path $scriptDir "FMGoalMusicInstaller.nsi"
    if (-not (Test-Path $nsi)) {
        Write-Log "NSIS script not found at $nsi — skipping installer creation."
    } else {
        Create-Installer -nsiPath $nsi
    }

    Write-Log "Build script finished successfully."
} catch {
    Write-Error "Exception: $($_.Exception.Message)"
    Write-Error $_.Exception.StackTrace
    # upload logs / leave artifacts for inspection
    Write-Log "vcpkg log saved to: $vcpkgLog"
    Write-Log "nsis log saved to: $nsisLog"
    Exit 1
}
