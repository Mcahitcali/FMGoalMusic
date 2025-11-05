Write-Host "🔧 Starting FM Goal Musics Windows build..." -ForegroundColor Cyan

# Ensure directories
if (-Not (Test-Path "build/windows")) { New-Item -ItemType Directory -Force -Path "build/windows" | Out-Null }

# Run cargo build
Write-Host "🚀 Running Cargo build..."
$env:RUSTFLAGS = "-C codegen-units=1"
& cargo build --release --jobs 1

# Copy executable
Copy-Item "target\release\fm-goal-musics-gui.exe" "build\windows\fm-goal-musics-gui.exe" -Force

# Copy assets
if (Test-Path "assets") {
    Copy-Item "assets\*" "build\windows\" -Recurse -Force
}

# Copy icons if exist
if (Test-Path "resources\icons") {
    Copy-Item "resources\icons\*" "build\windows\" -Recurse -Force
}

# Copy Tesseract/Leptonica DLLs if installed system-wide
$TessPath = "C:\Program Files\Tesseract-OCR"
if (Test-Path $TessPath) {
    Write-Host "📦 Copying Tesseract DLLs..."
    Copy-Item "$TessPath\*.dll" "build\windows\" -Force
}

$LeptDlls = Get-ChildItem -Path "C:\Windows\System32" -Filter "liblept*.dll" -ErrorAction SilentlyContinue
if ($LeptDlls) {
    Write-Host "📦 Copying Leptonica DLLs..."
    Copy-Item $LeptDlls.FullName "build\windows\" -Force
}

Write-Host "✅ Build completed successfully!"
