param(
    [switch]$FailOnMissing,
    [switch]$InstallMissing
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:AutoInstallEnabled = [bool]$InstallMissing

$results = New-Object System.Collections.Generic.List[object]

function Add-Result {
    param(
        [string]$Name,
        [bool]$Success,
        [string]$Detail
    )

    $results.Add([pscustomobject]@{
        Check  = $Name
        Status = if ($Success) { 'OK' } else { 'MISSING' }
        Detail = $Detail
    })

    $prefix = if ($Success) { '[OK]' } else { '[MISSING]' }
    $color  = if ($Success) { 'Green' } else { 'Red' }
    Write-Host ("{0} {1} - {2}" -f $prefix, $Name, $Detail) -ForegroundColor $color
}

function Select-Detail {
    param(
        [bool]$Condition,
        [object]$WhenTrue,
        [object]$WhenFalse
    )

    if ($Condition) {
        return $WhenTrue
    }
    return $WhenFalse
}

Write-Host "=== FM Goal Musics Windows Runner Checklist ===" -ForegroundColor Cyan
if ($InstallMissing) {
    Write-Host "Auto-install mode ENABLED (will attempt to install missing dependencies)." -ForegroundColor Yellow
}
Write-Host "Computer: $env:COMPUTERNAME" -ForegroundColor Cyan
Write-Host "User: $env:USERNAME" -ForegroundColor Cyan
Write-Host "PowerShell: $($PSVersionTable.PSVersion)" -ForegroundColor Cyan
Write-Host "Working Directory: $(Get-Location)" -ForegroundColor Cyan
Write-Host "==============================================`n"

# Helper
function Test-Command {
    param([string]$Command)
    try {
        $null = Get-Command $Command -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

function Ensure-Winget {
    if (Test-Command 'winget') { return $true }
    Write-Warning "winget CLI not available. Install App Installer from Microsoft Store to enable automated installs."
    return $false
}

function Install-WithWinget {
    param(
        [string]$DisplayName,
        [string]$PackageId,
        [string[]]$ExtraArgs
    )

    if (-not $script:AutoInstallEnabled) { return $false }
    if (-not (Ensure-Winget)) { return $false }

    Write-Host "Attempting winget install for $DisplayName ($PackageId) ..." -ForegroundColor Yellow
    $args = @('install','--id', $PackageId,'-e','--source','winget')
    if ($ExtraArgs) { $args += $ExtraArgs }
    try {
        $proc = Start-Process -FilePath 'winget' -ArgumentList $args -Wait -PassThru
        if ($proc.ExitCode -eq 0) {
            Write-Host "winget install for $DisplayName succeeded." -ForegroundColor Green
            return $true
        }
        Write-Warning "winget install for $DisplayName exited with $($proc.ExitCode)."
    } catch {
        Write-Warning "winget install for $DisplayName failed: $($_.Exception.Message)"
    }
    return $false
}

function Install-VSBuildTools {
    if (-not $script:AutoInstallEnabled) { return $false }
    $url = 'https://aka.ms/vs/17/release/vs_BuildTools.exe'
    $downloadPath = Join-Path $env:TEMP 'vs_buildtools.exe'
    try {
        Write-Host "Downloading Visual Studio Build Tools bootstrapper ..." -ForegroundColor Yellow
        Invoke-WebRequest -Uri $url -OutFile $downloadPath -UseBasicParsing
        $arguments = @(
            '--quiet','--norestart','--nocache',
            '--installPath','C:\BuildTools',
            '--add','Microsoft.VisualStudio.Workload.VCTools',
            '--add','Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
            '--add','Microsoft.VisualStudio.Component.Windows10SDK.22621',
            '--includeRecommended'
        )
        Write-Host "Installing Visual Studio Build Tools (this can take several minutes)..." -ForegroundColor Yellow
        $proc = Start-Process -FilePath $downloadPath -ArgumentList $arguments -Wait -PassThru
        if ($proc.ExitCode -eq 0) {
            Write-Host "Visual Studio Build Tools installation complete." -ForegroundColor Green
            return $true
        }
        Write-Warning "Visual Studio Build Tools installer exited with $($proc.ExitCode)."
    } catch {
        Write-Warning "Failed to install Visual Studio Build Tools: $($_.Exception.Message)"
    }
    return $false
}

function Ensure-RustTarget {
    if (-not (Test-Command 'rustup')) { return $false }
    try {
        Write-Host "Ensuring rustup target x86_64-pc-windows-msvc ..." -ForegroundColor Yellow
        $proc = Start-Process -FilePath 'rustup' -ArgumentList @('target','add','x86_64-pc-windows-msvc') -Wait -PassThru
        return ($proc.ExitCode -eq 0)
    } catch {
        Write-Warning "rustup target add failed: $($_.Exception.Message)"
        return $false
    }
}

function Install-Vcpkg {
    param(
        [string]$Root
    )

    if (-not $script:AutoInstallEnabled) { return $false }
    try {
        if (-not (Test-Command 'git')) {
            Write-Warning "Git is required to clone vcpkg. Install Git first."
            return $false
        }
        if (-not (Test-Path $Root)) {
            Write-Host "Cloning vcpkg into $Root ..." -ForegroundColor Yellow
            git clone https://github.com/microsoft/vcpkg $Root
        }
        $bootstrap = Join-Path $Root 'bootstrap-vcpkg.bat'
        Write-Host "Bootstrapping vcpkg ..." -ForegroundColor Yellow
        $proc = Start-Process -FilePath $bootstrap -ArgumentList @() -Wait -PassThru -WorkingDirectory $Root
        if ($proc.ExitCode -eq 0) {
            Write-Host "vcpkg bootstrap complete." -ForegroundColor Green
            return $true
        }
        Write-Warning "vcpkg bootstrap exited with $($proc.ExitCode)."
    } catch {
        Write-Warning "Failed to install vcpkg: $($_.Exception.Message)"
    }
    return $false
}

function Install-VcpkgPackages {
    param(
        [string]$VcpkgExe,
        [string]$Triplet,
        [string[]]$Packages
    )

    foreach ($pkg in $Packages) {
        $pattern = "{0}:{1}" -f $pkg,$Triplet
        $pkgSpec = $pattern
        Write-Host "Installing vcpkg package $pattern ..." -ForegroundColor Yellow
        $proc = Start-Process -FilePath $VcpkgExe -ArgumentList @('install',$pkgSpec) -Wait -PassThru
        if ($proc.ExitCode -ne 0) {
            Write-Warning "Failed to install $pattern (exit $($proc.ExitCode))."
            return $false
        }
    }
    return $true
}

# 1. Visual Studio Build Tools / MSVC
$vswhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
if (-not (Test-Path $vswhere)) {
    $vswhere = 'C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe'
}
$vsArgs = @('-latest','-requires','Microsoft.VisualStudio.Component.VC.Tools.x86.x64','-requires','Microsoft.VisualStudio.Component.Windows10SDK','-format','json')
$vsArgsLite = @('-latest','-requires','Microsoft.VisualStudio.Component.VC.Tools.x86.x64','-format','json')

function Get-VSInstances {
    param(
        [string]$VswherePath,
        [string[]]$Arguments
    )

    if (-not (Test-Path $VswherePath)) { return @() }
    try {
        $raw = & $VswherePath @Arguments
        if ([string]::IsNullOrWhiteSpace($raw)) { return @() }
        $parsed = $raw | ConvertFrom-Json
        if ($null -eq $parsed) { return @() }
        if ($parsed -is [System.Array]) { return $parsed }
        return @($parsed)
    } catch {
        Write-Warning "vswhere call failed: $($_.Exception.Message)"
        return @()
    }
}

if (Test-Path $vswhere) {
    $vsInstances = @(Get-VSInstances -VswherePath $vswhere -Arguments $vsArgs)
    if ($vsInstances.Count -gt 0) {
        $instance = $vsInstances[0]
        $vcvars = Join-Path $instance.installationPath 'VC\Auxiliary\Build\vcvars64.bat'
        $hasVc = Test-Path $vcvars
        if (-not $hasVc -and $InstallMissing) {
            if (Install-VSBuildTools) {
                $vsInstances = @(Get-VSInstances -VswherePath $vswhere -Arguments $vsArgsLite)
                if ($vsInstances.Count -gt 0) {
                    $instance = $vsInstances[0]
                    $vcvars = Join-Path $instance.installationPath 'VC\Auxiliary\Build\vcvars64.bat'
                    $hasVc = Test-Path $vcvars
                }
            }
        }
        Add-Result -Name 'Visual Studio Build Tools (C++ workload)' -Success $hasVc -Detail (Select-Detail $hasVc "Found at $($instance.installationPath)" 'vcvars64.bat not found')
    } else {
        if ($InstallMissing -and (Install-VSBuildTools)) {
            $vsInstances = @(Get-VSInstances -VswherePath $vswhere -Arguments $vsArgsLite)
            if ($vsInstances.Count -gt 0) {
                $instance = $vsInstances[0]
                $vcvars = Join-Path $instance.installationPath 'VC\Auxiliary\Build\vcvars64.bat'
                $hasVc = Test-Path $vcvars
                Add-Result -Name 'Visual Studio Build Tools (C++ workload)' -Success $hasVc -Detail (Select-Detail $hasVc "Found at $($instance.installationPath)" 'vcvars64.bat not found')
            } else {
                Add-Result -Name 'Visual Studio Build Tools (C++ workload)' -Success $false -Detail 'Build Tools install attempted but vswhere still missing components'
            }
        } else {
            Add-Result -Name 'Visual Studio Build Tools (C++ workload)' -Success $false -Detail 'vswhere found but required components missing'
        }
    }
} else {
    if ($InstallMissing -and (Install-VSBuildTools)) {
        $vsInstances = @(Get-VSInstances -VswherePath $vswhere -Arguments $vsArgsLite)
        if ($vsInstances.Count -gt 0) {
            $instance = $vsInstances[0]
            $vcvars = Join-Path $instance.installationPath 'VC\Auxiliary\Build\vcvars64.bat'
            $hasVc = Test-Path $vcvars
            Add-Result -Name 'Visual Studio Build Tools (C++ workload)' -Success $hasVc -Detail (if ($hasVc) { "Found at $($instance.installationPath)" } else { 'vcvars64.bat not found' })
        } else {
            Add-Result -Name 'Visual Studio Build Tools (C++ workload)' -Success $false -Detail 'VS installer ran but vswhere still unavailable'
        }
    } else {
        Add-Result -Name 'Visual Studio Build Tools (C++ workload)' -Success $false -Detail 'vswhere.exe not found'
    }
}

# 2. Windows 10 SDK
try {
    $kitsRoot = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots').KitsRoot10
    $sdkInclude = Join-Path $kitsRoot 'Include'
    $sdkOk = Test-Path $sdkInclude
    if (-not $sdkOk -and $InstallMissing) {
        if (Install-WithWinget -DisplayName 'Windows 10 SDK' -PackageId 'Microsoft.WindowsSDK.10.0.22621') {
            $kitsRoot = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots').KitsRoot10
            $sdkInclude = Join-Path $kitsRoot 'Include'
            $sdkOk = Test-Path $sdkInclude
        }
    }
    Add-Result -Name 'Windows 10 SDK 10.0.19041+' -Success $sdkOk -Detail ("Include path: {0}" -f $sdkInclude)
} catch {
    Add-Result -Name 'Windows 10 SDK 10.0.19041+' -Success $false -Detail 'Registry key missing'
}

# 3. Rust toolchain
if (Test-Command 'rustc' -and Test-Command 'cargo') {
    $rustcVersion = (rustc --version)
} elseif ($InstallMissing -and (Install-WithWinget -DisplayName 'Rust (rustup)' -PackageId 'Rustlang.Rustup')) {
    $rustcVersion = (rustc --version)
} else {
    $rustcVersion = $null
}
Add-Result -Name 'Rust (rustc)' -Success ([bool]$rustcVersion) -Detail (Select-Detail ([bool]$rustcVersion) $rustcVersion 'rustc/cargo not on PATH')

if (Test-Command 'rustup') {
    $targets = (rustup target list --installed)
    $hasMsvc = $targets -contains 'x86_64-pc-windows-msvc'
    if (-not $hasMsvc -and $InstallMissing) {
        $hasMsvc = Ensure-RustTarget
        if ($hasMsvc) { $targets = (rustup target list --installed) }
    }
    Add-Result -Name 'Rust target x86_64-pc-windows-msvc' -Success $hasMsvc -Detail (Select-Detail $hasMsvc ($targets -join ', ') 'Target missing (run rustup target add x86_64-pc-windows-msvc)')
} else {
    Add-Result -Name 'Rustup' -Success $false -Detail 'rustup not found'
}

# 4. Git + Bash
$gitOk = Test-Command 'git'
if (-not $gitOk -and $InstallMissing) {
    if (Install-WithWinget -DisplayName 'Git' -PackageId 'Git.Git') {
        $gitOk = Test-Command 'git'
    }
}
Add-Result -Name 'Git CLI' -Success $gitOk -Detail (Select-Detail $gitOk (git --version) 'git not found')

$bashPaths = @(
    'C:\Program Files\Git\usr\bin\bash.exe',
    'C:\Program Files\Git\bin\bash.exe',
    'C:\Program Files (x86)\Git\usr\bin\bash.exe',
    'C:\Program Files (x86)\Git\bin\bash.exe'
)
$bashLocation = $bashPaths | Where-Object { Test-Path $_ } | Select-Object -First 1
Add-Result -Name 'Git Bash (bash.exe)' -Success ([bool]$bashLocation) -Detail (Select-Detail ([bool]$bashLocation) $bashLocation 'Install Git for Windows with Bash')

# 5. LLVM / Clang
$libClangPath = $env:LIBCLANG_PATH
if (-not $libClangPath) { $libClangPath = 'C:\Program Files\LLVM\bin' }
$clangExe = Join-Path $libClangPath 'clang.exe'
if (-not (Test-Path $clangExe) -and $InstallMissing) {
    if (Install-WithWinget -DisplayName 'LLVM' -PackageId 'LLVM.LLVM') {
        $clangExe = 'C:\Program Files\LLVM\bin\clang.exe'
    }
}
$hasClang = Test-Path $clangExe
Add-Result -Name 'LLVM/Clang' -Success $hasClang -Detail (Select-Detail $hasClang "Found clang at $clangExe" 'clang.exe not found; install LLVM and set LIBCLANG_PATH')

# 6. NSIS
$nsisPaths = @(
    'C:\Program Files (x86)\NSIS\makensis.exe',
    'C:\Program Files\NSIS\makensis.exe'
)
$nsisExe = $nsisPaths | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $nsisExe -and $InstallMissing) {
    if (Install-WithWinget -DisplayName 'NSIS' -PackageId 'NSIS.NSIS') {
        $nsisExe = $nsisPaths | Where-Object { Test-Path $_ } | Select-Object -First 1
    }
}
Add-Result -Name 'NSIS (makensis)' -Success ([bool]$nsisExe) -Detail (Select-Detail ([bool]$nsisExe) $nsisExe 'Install NSIS from nsis.sourceforge.io')

# 7. vcpkg + packages
$vcpkgRoot = if ($env:VCPKG_ROOT) { $env:VCPKG_ROOT } else { 'C:\vcpkg' }
$vcpkgExe = Join-Path $vcpkgRoot 'vcpkg.exe'
if (-not (Test-Path $vcpkgExe) -and $InstallMissing) {
    Install-Vcpkg -Root $vcpkgRoot | Out-Null
}

if (Test-Path $vcpkgExe) {
    Add-Result -Name 'vcpkg.exe' -Success $true -Detail $vcpkgExe
    try {
        $triplet = if ($env:VCPKG_DEFAULT_TRIPLET) { $env:VCPKG_DEFAULT_TRIPLET } else { 'x64-windows-static' }
        $pkgList = & $vcpkgExe list
        $required = @('leptonica','tesseract','zlib','libpng','libtiff','libjpeg-turbo')
        $missing = @()
        foreach ($pkg in $required) {
            $pattern = "{0}:{1}" -f $pkg, $triplet

            # vcpkg list satırlarını eşleştir ve boolean'a indir
            $matches = $pkgList -match [regex]::Escape($pattern)
            $isInstalled = $false
            if ($matches) { $isInstalled = $true }

            if (-not $isInstalled) { $missing += $pkg }
            if (-not $isInstalled -and $InstallMissing) {
                if (Install-VcpkgPackages -VcpkgExe $vcpkgExe -Triplet $triplet -Packages @($pkg)) {
                    $pkgList = & $vcpkgExe list
                    $matches = $pkgList -match [regex]::Escape($pattern)
                    $isInstalled = $false
                    if ($matches) { $isInstalled = $true }
                }
            }

            Add-Result -Name ("vcpkg package: {0}" -f $pattern) -Success $isInstalled -Detail (Select-Detail $isInstalled 'Installed' ('Missing - run vcpkg install ' + $pattern))
        }
    } catch {
        Add-Result -Name 'vcpkg list' -Success $false -Detail $_.Exception.Message
    }
} else {
    Add-Result -Name 'vcpkg.exe' -Success $false -Detail "Expected at $vcpkgExe"
}

# 8. WebView2 Runtime
try {
    $edgeKey = 'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients'
    $webViewKey = Get-ChildItem $edgeKey | Where-Object { $_.GetValue('name') -eq 'Microsoft Edge WebView2 Runtime' }
    $webViewVersion = $webViewKey.GetValue('pv')
    if (-not $webViewVersion -and $InstallMissing) {
        if (Install-WithWinget -DisplayName 'WebView2 Runtime' -PackageId 'Microsoft.EdgeWebView2Runtime') {
            $webViewKey = Get-ChildItem $edgeKey | Where-Object { $_.GetValue('name') -eq 'Microsoft Edge WebView2 Runtime' }
            $webViewVersion = $webViewKey.GetValue('pv')
        }
    }
    Add-Result -Name 'WebView2 Runtime' -Success ([bool]$webViewVersion) -Detail (Select-Detail ([bool]$webViewVersion) "Version $webViewVersion" 'Not found')
} catch {
    Add-Result -Name 'WebView2 Runtime' -Success $false -Detail 'Registry entry not found'
}

# 9. Environment variables summary
Add-Result -Name 'ENV VCPKG_ROOT' -Success ([bool]$env:VCPKG_ROOT) -Detail (Select-Detail ([bool]$env:VCPKG_ROOT) $env:VCPKG_ROOT 'Not set')
Add-Result -Name 'ENV LIBCLANG_PATH' -Success ([bool]$env:LIBCLANG_PATH) -Detail (Select-Detail ([bool]$env:LIBCLANG_PATH) $env:LIBCLANG_PATH 'Not set')

Write-Host "`n=== Summary ===" -ForegroundColor Cyan
$results | Format-Table -AutoSize

if ($FailOnMissing -and ($results | Where-Object { $_.Status -ne 'OK' })) {
    Write-Error "One or more checks failed. See summary above."
    exit 1
}

Write-Host "`nChecklist finished." -ForegroundColor Cyan
