# Qt Environment Setup for MyMe
# Run from repo root: . .\scripts\setup-qt-env.ps1
param(
    [string]$QtPath = "C:\Qt"
)

Write-Host "MyMe - Qt Environment Setup" -ForegroundColor Cyan
Write-Host "===========================" -ForegroundColor Cyan
Write-Host ""

# Find Qt installation
Write-Host "Searching for Qt installation in: $QtPath" -ForegroundColor Yellow

if (-not (Test-Path $QtPath)) {
    Write-Host "ERROR: Qt directory not found at $QtPath" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install Qt 6.x first:" -ForegroundColor Yellow
    Write-Host "1. Download from: https://www.qt.io/download-qt-installer" -ForegroundColor White
    Write-Host "2. Install Qt 6.7 or later with MSVC 64-bit components" -ForegroundColor White
    Write-Host "3. Run this script again" -ForegroundColor White
    Write-Host ""
    Write-Host "Or specify custom path:" -ForegroundColor Yellow
    Write-Host "  . .\scripts\setup-qt-env.ps1 -QtPath 'C:\path\to\qt'" -ForegroundColor White
    exit 1
}

$qtVersions = Get-ChildItem $QtPath -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '^\d+\.\d+\.\d+$' } |
    Sort-Object Name -Descending

if ($qtVersions.Count -eq 0) {
    Write-Host "ERROR: No Qt version directories found in $QtPath" -ForegroundColor Red
    Write-Host ""
    Write-Host "Expected directory structure:" -ForegroundColor Yellow
    Write-Host "  C:\Qt\6.7.0\msvc2019_64\" -ForegroundColor White
    Write-Host ""
    Write-Host "Please install Qt 6.x. See QT_INSTALLATION.md for details." -ForegroundColor Yellow
    exit 1
}

$latestQt = $qtVersions[0]
$qtVersionPath = Join-Path $QtPath $latestQt.Name

Write-Host "Found Qt version: $($latestQt.Name)" -ForegroundColor Green
Write-Host ""

# Find MSVC compiler version
Write-Host "Looking for MSVC 64-bit build..." -ForegroundColor Yellow

$msvcDirs = Get-ChildItem $qtVersionPath -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '^msvc\d+_64$' } |
    Sort-Object Name -Descending

if ($msvcDirs.Count -eq 0) {
    Write-Host "ERROR: No MSVC 64-bit build found in Qt installation" -ForegroundColor Red
    Write-Host ""
    Write-Host "Available directories in $qtVersionPath :" -ForegroundColor Yellow
    Get-ChildItem $qtVersionPath -Directory | ForEach-Object {
        Write-Host "  - $($_.Name)" -ForegroundColor White
    }
    Write-Host ""
    Write-Host "Please ensure you installed the MSVC 64-bit component." -ForegroundColor Yellow
    Write-Host "Reinstall Qt and select: Qt 6.x -> MSVC 2019 64-bit" -ForegroundColor White
    exit 1
}

$qtCompilerPath = $msvcDirs[0].FullName
$qtCmakePath = Join-Path $qtCompilerPath "lib\cmake\Qt6"
$qtBinPath = Join-Path $qtCompilerPath "bin"

Write-Host "Found: $($msvcDirs[0].Name)" -ForegroundColor Green
Write-Host "Qt Path: $qtCompilerPath" -ForegroundColor White
Write-Host ""

# Verify critical files exist
$qmakePath = Join-Path $qtBinPath "qmake.exe"
if (-not (Test-Path $qmakePath)) {
    Write-Host "WARNING: qmake.exe not found at $qmakePath" -ForegroundColor Yellow
    Write-Host "Qt installation may be incomplete." -ForegroundColor Yellow
}

if (-not (Test-Path $qtCmakePath)) {
    Write-Host "WARNING: CMake configuration not found at $qtCmakePath" -ForegroundColor Yellow
    Write-Host "Qt installation may be incomplete." -ForegroundColor Yellow
}

# Set environment variables for current session
Write-Host "Setting environment variables for current session..." -ForegroundColor Yellow

$env:CMAKE_PREFIX_PATH = $qtCompilerPath
$env:Qt6_DIR = $qtCmakePath
$env:PATH = "$qtBinPath;$env:PATH"
$env:QMAKE = $qmakePath

Write-Host "  CMAKE_PREFIX_PATH = $qtCompilerPath" -ForegroundColor White
Write-Host "  Qt6_DIR = $qtCmakePath" -ForegroundColor White
Write-Host "  Added to PATH: $qtBinPath" -ForegroundColor White
Write-Host ""

# Verify qmake works
Write-Host "Verifying Qt installation..." -ForegroundColor Yellow

try {
    $qmakeVersion = & qmake --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Qt verified successfully!" -ForegroundColor Green
        Write-Host $qmakeVersion -ForegroundColor White
    } else {
        Write-Host "WARNING: qmake returned an error" -ForegroundColor Yellow
    }
} catch {
    Write-Host "WARNING: Could not run qmake" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "===========================" -ForegroundColor Cyan
Write-Host "Qt environment configured!" -ForegroundColor Green
Write-Host "===========================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This configuration is temporary (current session only)." -ForegroundColor Yellow
Write-Host ""
Write-Host "To make permanent, run these commands:" -ForegroundColor Yellow
Write-Host ""
Write-Host "[System.Environment]::SetEnvironmentVariable('CMAKE_PREFIX_PATH', '$qtCompilerPath', 'User')" -ForegroundColor Cyan
Write-Host "[System.Environment]::SetEnvironmentVariable('Qt6_DIR', '$qtCmakePath', 'User')" -ForegroundColor Cyan
Write-Host ""
Write-Host "Or use Windows System Properties -> Environment Variables" -ForegroundColor White
Write-Host ""
Write-Host "You can now build MyMe:" -ForegroundColor Green
Write-Host "  .\scripts\build.ps1   (full Qt app) or .\scripts\build-rust.ps1 (Rust only)" -ForegroundColor White
Write-Host "  # or" -ForegroundColor Gray
Write-Host "  cargo build --release" -ForegroundColor White
Write-Host ""
