# MyMe Build Script - Uses Visual Studio Developer Environment
# This script launches a Developer Command Prompt and runs the build

Write-Host "MyMe - Visual Studio Build Script" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

# Find Visual Studio installation
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"

if (-not (Test-Path $vswhere)) {
    Write-Host "ERROR: Visual Studio not found." -ForegroundColor Red
    Write-Host "Please install Visual Studio 2019 or later with C++ development tools." -ForegroundColor Yellow
    exit 1
}

$vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath

if ([string]::IsNullOrEmpty($vsPath)) {
    Write-Host "ERROR: Visual Studio with C++ tools not found." -ForegroundColor Red
    Write-Host "Please install 'Desktop development with C++' workload in Visual Studio." -ForegroundColor Yellow
    exit 1
}

Write-Host "Found Visual Studio: $vsPath" -ForegroundColor Green
Write-Host ""

# Path to VsDevCmd.bat
$vsDevCmd = Join-Path $vsPath "Common7\Tools\VsDevCmd.bat"

if (-not (Test-Path $vsDevCmd)) {
    Write-Host "ERROR: VsDevCmd.bat not found at: $vsDevCmd" -ForegroundColor Red
    exit 1
}

Write-Host "Initializing Visual Studio Developer Environment..." -ForegroundColor Yellow
Write-Host "Configuring Qt environment..." -ForegroundColor Yellow
Write-Host "Running: cargo build --release" -ForegroundColor Yellow
Write-Host ""

# Set Qt environment variables
$qtPath = "C:\Qt\6.10.1\msvc2022_64"
$qtCmakePath = "$qtPath\lib\cmake\Qt6"

# Run cargo build in VS Developer environment with Qt variables
$buildCommand = "`"$vsDevCmd`" && set CMAKE_PREFIX_PATH=$qtPath && set Qt6_DIR=$qtCmakePath && set QMAKE=$qtPath\bin\qmake.exe && cargo build --release"

$process = Start-Process -FilePath "cmd.exe" -ArgumentList "/c", $buildCommand -WorkingDirectory $PSScriptRoot -NoNewWindow -Wait -PassThru

if ($process.ExitCode -eq 0) {
    Write-Host ""
    Write-Host "==================================" -ForegroundColor Cyan
    Write-Host "BUILD SUCCESS!" -ForegroundColor Green
    Write-Host "==================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Binary location: target\release\myme.exe" -ForegroundColor White
} else {
    Write-Host ""
    Write-Host "==================================" -ForegroundColor Cyan
    Write-Host "BUILD FAILED" -ForegroundColor Red
    Write-Host "==================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Exit code: $($process.ExitCode)" -ForegroundColor Red
    exit $process.ExitCode
}
