# ========================================

# FMGoalMusic - Auto Build & Package Script (Final Versioned)

# ========================================

# Read version from Cargo.toml

$versionLine = Get-Content ".\Cargo.toml" | Where-Object { $_ -match '^version\s*=' }

if ($versionLine -match '"([^"]+)"') {

$version = $matches[1]

} else {

$version = "0.0.0"

}

# Get date

$today = Get-Date -Format "yyyy-MM-dd"

# Define output filename

$installerName = "FMGoalMusicInstaller_${today}_v${version}.exe"

Write-Host "---------------------------------------------"

Write-Host " Building FM Goal Musics (release mode)..."

Write-Host "---------------------------------------------"

cargo build --release

if ($LASTEXITCODE -ne 0) {

Write-Host "Build failed. Exiting."

exit 1

}

Write-Host "`nChecking tessdata directory..."

$src = Get-Location

$dst = "$src\target\release"

# Ensure tessdata exists by copying from system if missing

if (-not (Test-Path "$src\tessdata")) {

if (Test-Path "C:\Program Files\Tesseract-OCR\tessdata") {

Write-Host "tessdata not found in project. Copying from system Tesseract..."

New-Item -ItemType Directory -Force -Path "$src\tessdata" | Out-Null

Copy-Item "C:\Program Files\Tesseract-OCR\tessdata\*" "$src\tessdata" -Recurse -Force

} else {

Write-Host "Warning: tessdata not found in system. OCR may fail."

}

} else {

Write-Host "tessdata directory already exists in project."

}

Write-Host "`nCopying static folders to release directory..."

$folders = @("config", "assets", "tessdata")

foreach ($f in $folders) {

if (Test-Path "$src\$f") {

Write-Host "Copying $f ..."

# Create destination folder if missing

New-Item -ItemType Directory -Force -Path "$dst\$f" | Out-Null

# Copy only contents, not the folder itself

Copy-Item "$src\$f\*" "$dst\$f" -Recurse -Force

} else {

Write-Host "$f folder not found, skipping..."

}

}

Write-Host "`nRunning NSIS installer build..."

& "C:\Program Files (x86)\NSIS\makensis.exe" ".\FMGoalMusicInstaller.nsi"

if ($LASTEXITCODE -eq 0) {

Write-Host "`n---------------------------------------------"

Write-Host " Installer built successfully!"

$outputPath = Join-Path $src $installerName

Move-Item -Force "$src\FMGoalMusicInstaller.exe" $outputPath

Write-Host " Output: $outputPath"

Write-Host "---------------------------------------------"

} else {

Write-Host "`nNSIS build failed. Check for missing files or syntax errors."

}