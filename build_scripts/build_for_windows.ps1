<#
FM Goal Musics - Windows Setup & Build (robust for GitHub Actions hosted runners)
Drop-in replacement for build_scripts/build_for_windows.ps1
Writes logs to C:\vcpkg-logs\ and stages payload into build/windows/
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# --- Paths
$scriptPath = $MyInvocation.MyCommand.Definition
$scriptDir  = Split-Path -Parent $scriptPath
$repoRoot   = Resolve-Path (Join-Path $scriptDir "..") | Select-Object -ExpandProperty Path

$vcpkgLogDir = "C:\vcpkg-logs"
if (!(Test-Path $vcpkgLogDir)) { New-Item -ItemType Directory -Path $vcpkgLogDir | Out-Null }
$vcpkgLog = Join-Path $vcpkgLogDir "vcpkg_install.log"
$chocoLog  = Join-Path $vcpkgLogDir "choco_install.log"

function Fail($msg) {
    Write-Host "[ERROR] $msg" -ForegroundColor Red
    throw $msg
}

function Run-Proc($exe, [string[]]$args, $outFile) {
    Write-Host "Run-Proc: $exe $($args -join ' ')"
    if ($null -eq $outFile) { $outFile = Join-Path $env:TEMP ("runproc_" + ([Guid]::NewGuid().ToString()) + ".log") }
    if (Test-Path $outFile) { Remove-Item $outFile -Force -ErrorAction SilentlyContinue }
    $errFile = "${outFile}.err"
    if (Test-Path $errFile) { Remove-Item $errFile -Force -ErrorAction SilentlyContinue }

    # Start-Process requires different files for stdout/stderr on Windows; use separate files then merge.
    $proc = Start-Process -FilePath $exe -ArgumentList $args -NoNewWindow -RedirectStandardOutput $outFile -RedirectStandardError $errFile -PassThru -Wait

    # If process failed, show logs and merge for artifact
    if ($proc.ExitCode -ne 0) {
        Write-Host "--- Last 200 lines of stdout ($outFile) ---"
        if (Test-Path $outFile) { Get-Content $outFile -Tail 200 | ForEach-Object { Write-Host $_ } }
        Write-Host "--- Last 200 lines of stderr ($errFile) ---"
        if (Test-Path $errFile) { Get-Content $errFile -Tail 200 | ForEach-Object { Write-Host $_ } }

        if (Test-Path $errFile) { Get-Content $errFile | Add-Content $outFile }
        Fail "$exe failed (exit $($proc.ExitCode)). See $outFile (stderr appended to same file)."
    }

    # merge stderr into stdout for single-file artifacts and cleanup
    if (Test-Path $errFile) {
        Get-Content $errFile | Add-Content $outFile
        Remove-Item $errFile -Force -ErrorAction SilentlyContinue
    }
    return Get-Content $outFile -ErrorAction SilentlyContinue
}

# Print useful tool locations (avoid $t: parsing issues)
$tools = @("git","cmake","7z","pwsh","cl.exe")
foreach ($t in $tools) {
    $c = Get-Command $t -ErrorAction SilentlyContinue
    if ($c) { Write-Host "$($t): $($c.Path)" } else { Write-Host "Warning: $($t) not on PATH" }
}

# --- Import Visual Studio environment
$vsCandidates = @(
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat",
    "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\Common7\Tools\VsDevCmd.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\Common7\Tools\VsDevCmd.bat"
)
$vsDevCmd = $vsCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $vsDevCmd) { Fail "VsDevCmd.bat not found. Ensure Visual Studio Build Tools are installed on runner." }

Write-Host "Importing VS dev environment from: $vsDevCmd"
$tempBat = Join-Path $env:TEMP "vsenv_wrapper.bat"
$tempEnvOut = Join-Path $env:TEMP "vsenv_out.txt"
$batContent = "@echo off`r`ncall `"$vsDevCmd`" -host_arch=amd64 -arch=amd64`r`nset > `"$tempEnvOut`""
Set-Content -Path $tempBat -Value $batContent -Encoding ASCII

# Run wrapper
$cmdOut = Join-Path $vcpkgLogDir "vsdevcmd_out.txt"
Run-Proc -exe "cmd.exe" -args @("/c", "`"$tempBat`"") -outFile $cmdOut | Out-Null
if (-not (Test-Path $tempEnvOut)) { Fail "Failed to capture VS environment. Expected $tempEnvOut" }

# Load environment variables captured by bat into current process
Get-Content $tempEnvOut | ForEach-Object {
    if ($_ -match '^(.*?)=(.*)$') {
        $name = $matches[1]; $value = $matches[2]
        if ($name) {
            [System.Environment]::SetEnvironmentVariable($name, $value, "Process")
        }
    }
}
Write-Host "MSVC environment imported."
if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) { Write-Host "Warning: cl.exe not on PATH after VsDevCmd import" }

# --- Chocolatey best-effort installs ---
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Warning: choco not found. Script will attempt best-effort without choco (hosted runner normally has choco)."
}

if (-not (Get-Command makensis -ErrorAction SilentlyContinue) -and (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Installing NSIS via choco..."
    Run-Proc -exe "choco" -args @("install","nsis.install","-y","--no-progress") -outFile $chocoLog
    refreshenv | Out-Null
}

if (-not (Get-Command tesseract -ErrorAction SilentlyContinue) -and (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Tesseract via choco..."
    Run-Proc -exe "choco" -args @("install","tesseract","-y","--no-progress") -outFile $chocoLog
    refreshenv | Out-Null
}

# --- vcpkg bootstrap & install
$vcpkgRoot = Join-Path $env:USERPROFILE "vcpkg"
$vcpkgExe  = Join-Path $vcpkgRoot "vcpkg.exe"

if (-not (Test-Path $vcpkgExe)) {
    Write-Host "Cloning vcpkg into $vcpkgRoot..."
    if (Test-Path $vcpkgRoot) { Remove-Item -Recurse -Force $vcpkgRoot -ErrorAction SilentlyContinue }
    Run-Proc -exe "git" -args @("clone","--depth","1","https://github.com/microsoft/vcpkg.git",$vcpkgRoot) -outFile (Join-Path $vcpkgLogDir "git_clone_vcpkg.log")
    Push-Location $vcpkgRoot
    Write-Host "Bootstrapping vcpkg..."
    Run-Proc -exe ".\bootstrap-vcpkg.bat" -args @("-disableMetrics") -outFile $vcpkgLog
    Pop-Location
} else {
    Write-Host "vcpkg already present at $vcpkgRoot"
}

[System.Environment]::SetEnvironmentVariable("VCPKG_ROOT", $vcpkgRoot, "Process")
[System.Environment]::SetEnvironmentVariable("VCPKG_DEFAULT_TRIPLET", "x64-windows", "Process")
[System.Environment]::SetEnvironmentVariable("VCPKG_FEATURE_FLAGS", "manifests", "Process")

Write-Host "Installing vcpkg ports: leptonica, tesseract (triplet x64-windows) — logs -> $vcpkgLog"
Push-Location $vcpkgRoot
if (Test-Path $vcpkgLog) { Remove-Item $vcpkgLog -Force -ErrorAction SilentlyContinue }

$installArgs = @("install","leptonica","tesseract","--triplet","x64-windows","--clean-after-build","-v")
Run-Proc -exe $vcpkgExe -args $installArgs -outFile $vcpkgLog

$listOut = & $vcpkgExe list --triplet x64-windows 2>&1
Write-Host "vcpkg list (tail):"
$listOut | Select-Object -Last 40 | ForEach-Object { Write-Host $_ }

if ($listOut -notmatch "leptonica:.*x64-windows") { Fail "leptonica not found in vcpkg list; see $vcpkgLog" }
if ($listOut -notmatch "tesseract:.*x64-windows")  { Fail "tesseract not found in vcpkg list; see $vcpkgLog" }
Pop-Location
Write-Host "vcpkg: leptonica & tesseract installed."

# --- Build Rust release
Write-Host "[1/2] Building Rust (release, x86_64-pc-windows-msvc)"
Push-Location $repoRoot
Run-Proc -exe "cargo" -args @("build","--release","--target","x86_64-pc-windows-msvc") -outFile (Join-Path $vcpkgLogDir "cargo_build.log")
Pop-Location

$exeName = "fm-goal-musics-gui.exe"
$exeRel  = Join-Path $repoRoot "target\x86_64-pc-windows-msvc\release\$exeName"
if (-not (Test-Path $exeRel)) { Fail "Built binary not found: $exeRel" }

# --- Stage runtime
$buildDir = Join-Path $repoRoot "build\windows"
if (Test-Path $buildDir) { Remove-Item -Recurse -Force $buildDir -ErrorAction SilentlyContinue }
New-Item -ItemType Directory -Path $buildDir | Out-Null
Copy-Item $exeRel -Destination $buildDir -Force
Write-Host "Copied binary to $buildDir"

# include assets if present
$maybeAssets = @("config","assets","README.md","LICENSE")
foreach ($it in $maybeAssets) {
    $src = Join-Path $repoRoot $it
    if (Test-Path $src) {
        Copy-Item $src -Destination $buildDir -Recurse -Force
        Write-Host "Included: $it"
    }
}

# include vcpkg runtime DLLs if present
$vcpkgBin = Join-Path $vcpkgRoot "installed\x64-windows\bin"
if (Test-Path $vcpkgBin) {
    Copy-Item (Join-Path $vcpkgBin "*.dll") -Destination $buildDir -Force -ErrorAction SilentlyContinue
    Write-Host "Included vcpkg DLLs from $vcpkgBin"
} else {
    Write-Host "Warning: vcpkg bin folder not found; runtime DLLs may be missing."
}

# include tessdata if present
$tessdataSrc = Join-Path $vcpkgRoot "installed\x64-windows\share\tesseract\tessdata"
if (Test-Path $tessdataSrc) {
    Copy-Item $tessdataSrc -Destination (Join-Path $buildDir "tessdata") -Recurse -Force
    Write-Host "Included tessdata"
} else {
    Write-Host "Warning: tessdata not found in vcpkg installed tree."
}

Write-Host "[2/2] Build staging complete — payload is in: $buildDir"
Write-Host "All done."
