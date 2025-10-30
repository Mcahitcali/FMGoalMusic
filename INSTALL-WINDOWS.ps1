# FM Goal Musics - One-Click Windows Installer (PowerShell)
# This script installs Rust and builds the application automatically

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  FM Goal Musics - Auto Installer" -ForegroundColor Cyan  
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This installer will:" -ForegroundColor Yellow
Write-Host "1. Install Rust (if not installed)" -ForegroundColor White
Write-Host "2. Build the application" -ForegroundColor White
Write-Host "3. Create ready-to-use package" -ForegroundColor White
Write-Host ""

# Check if running as administrator
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "⚠️  Note: Running without administrator privileges" -ForegroundColor Yellow
    Write-Host "   If installation fails, right-click and 'Run as administrator'" -ForegroundColor Yellow
    Write-Host ""
}

# Check if Rust is installed
Write-Host "🔍 Checking for Rust installation..." -ForegroundColor Blue
try {
    $rustVersion = rustc --version 2>$null
    if ($rustVersion) {
        Write-Host "✅ Rust is already installed: $rustVersion" -ForegroundColor Green
    }
} catch {
    Write-Host "❌ Rust not found. Installing Rust..." -ForegroundColor Red
    Write-Host ""
    
    Write-Host "📥 Downloading Rust installer..." -ForegroundColor Blue
    
    # Download Rust installer
    $installerPath = "$env:TEMP\rustup-init.exe"
    try {
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $installerPath
        Write-Host "✅ Download completed" -ForegroundColor Green
    } catch {
        Write-Host "❌ Failed to download Rust installer: $_" -ForegroundColor Red
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    Write-Host "📦 Installing Rust..." -ForegroundColor Blue
    Write-Host "This may take a few minutes..." -ForegroundColor Yellow
    
    # Install Rust silently
    try {
        Start-Process -FilePath $installerPath -ArgumentList "-y", "--default-toolchain", "stable" -Wait -NoNewWindow
        Write-Host "✅ Rust installed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "❌ Rust installation failed: $_" -ForegroundColor Red
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
    
    # Clean up installer
    Remove-Item $installerPath -Force -ErrorAction SilentlyContinue
    
    # Refresh PATH for current session
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}

# Refresh environment variables
Write-Host "🔄 Refreshing environment..." -ForegroundColor Blue
$env:PATH += ";$env:USERPROFILE\.cargo\bin"

# Verify Rust is working
Write-Host "🔧 Verifying Rust installation..." -ForegroundColor Blue
try {
    $rustVersion = rustc --version
    Write-Host "✅ Rust verification successful: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust verification failed" -ForegroundColor Red
    Write-Host "Please restart PowerShell and run this installer again" -ForegroundColor Yellow
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

Write-Host ""
Write-Host "🏗️  Building FM Goal Musics..." -ForegroundColor Blue
Write-Host "This may take 10-15 minutes on first build..." -ForegroundColor Yellow
Write-Host ""

# Run the build script
if (Test-Path "build_windows.bat") {
    try {
        & cmd.exe /c "build_windows.bat"
        if ($LASTEXITCODE -ne 0) {
            throw "Build script failed with exit code $LASTEXITCODE"
        }
        Write-Host "✅ Build completed successfully!" -ForegroundColor Green
    } catch {
        Write-Host "❌ Build failed: $_" -ForegroundColor Red
        Write-Host "Press any key to exit..."
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        exit 1
    }
} else {
    Write-Host "❌ build_windows.bat not found" -ForegroundColor Red
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  ✅ INSTALLATION COMPLETE!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "📍 Your application is ready in:" -ForegroundColor Yellow
Write-Host "   build\windows\fm-goal-musics-gui.exe" -ForegroundColor White
Write-Host ""
Write-Host "🎮 To run the application:" -ForegroundColor Yellow
Write-Host "   1. Navigate to build\windows\" -ForegroundColor White
Write-Host "   2. Double-click fm-goal-musics-gui.exe" -ForegroundColor White
Write-Host ""
Write-Host "📦 Or share the ZIP file with others:" -ForegroundColor Yellow
Write-Host "   build\windows\FM-Goal-Musics-Windows-*.zip" -ForegroundColor White
Write-Host ""
Write-Host "🎉 Enjoy your goal celebration music!" -ForegroundColor Green
Write-Host ""

# Ask if user wants to run the app
$choice = Read-Host "Do you want to run FM Goal Musics now? (Y/N)"
if ($choice -eq "Y" -or $choice -eq "y") {
    Write-Host "🚀 Starting FM Goal Musics..." -ForegroundColor Blue
    $exePath = "build\windows\fm-goal-musics-gui.exe"
    if (Test-Path $exePath) {
        Start-Process -FilePath $exePath
    } else {
        Write-Host "❌ Application not found at $exePath" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
