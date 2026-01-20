# Automated Visual Studio Linker Configuration for Rust
# This script detects Visual Studio installation and configures Cargo to use the correct linker

Write-Host "MyMe - Windows Linker Fix Script" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

# Function to find Visual Studio installation
function Find-VSInstallation {
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"

    if (-not (Test-Path $vswhere)) {
        Write-Host "ERROR: vswhere.exe not found. Visual Studio may not be installed." -ForegroundColor Red
        Write-Host "Please install Visual Studio 2019 or later with C++ development tools." -ForegroundColor Yellow
        return $null
    }

    # Find latest VS installation with C++ tools
    $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath

    if ([string]::IsNullOrEmpty($vsPath)) {
        Write-Host "ERROR: Visual Studio with C++ tools not found." -ForegroundColor Red
        Write-Host "Please install 'Desktop development with C++' workload in Visual Studio." -ForegroundColor Yellow
        return $null
    }

    return $vsPath
}

# Function to find VC tools version
function Find-VCToolsVersion {
    param([string]$vsPath)

    $vcToolsPath = Join-Path $vsPath "VC\Tools\MSVC"

    if (-not (Test-Path $vcToolsPath)) {
        Write-Host "ERROR: VC Tools directory not found at: $vcToolsPath" -ForegroundColor Red
        return $null
    }

    # Get the latest version
    $versions = Get-ChildItem $vcToolsPath | Sort-Object Name -Descending

    if ($versions.Count -eq 0) {
        Write-Host "ERROR: No VC Tools versions found." -ForegroundColor Red
        return $null
    }

    return $versions[0].Name
}

# Function to find Windows SDK version
function Find-WindowsSDKVersion {
    $sdkPath = "C:\Program Files (x86)\Windows Kits\10\Lib"

    if (-not (Test-Path $sdkPath)) {
        Write-Host "ERROR: Windows SDK not found at: $sdkPath" -ForegroundColor Red
        return $null
    }

    # Get the latest SDK version
    $versions = Get-ChildItem $sdkPath | Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } | Sort-Object Name -Descending

    if ($versions.Count -eq 0) {
        Write-Host "ERROR: No Windows SDK versions found." -ForegroundColor Red
        return $null
    }

    return $versions[0].Name
}

# Main execution
Write-Host "Step 1: Detecting Visual Studio installation..." -ForegroundColor Yellow
$vsPath = Find-VSInstallation

if ($null -eq $vsPath) {
    Write-Host ""
    Write-Host "SOLUTION: Install Visual Studio 2019 or later" -ForegroundColor Cyan
    Write-Host "1. Download from: https://visualstudio.microsoft.com/" -ForegroundColor White
    Write-Host "2. Select 'Desktop development with C++' workload" -ForegroundColor White
    Write-Host "3. Run this script again after installation" -ForegroundColor White
    exit 1
}

Write-Host "Found: $vsPath" -ForegroundColor Green
Write-Host ""

Write-Host "Step 2: Detecting VC Tools version..." -ForegroundColor Yellow
$vcVersion = Find-VCToolsVersion -vsPath $vsPath

if ($null -eq $vcVersion) {
    exit 1
}

Write-Host "Found: $vcVersion" -ForegroundColor Green
Write-Host ""

Write-Host "Step 3: Detecting Windows SDK version..." -ForegroundColor Yellow
$sdkVersion = Find-WindowsSDKVersion

if ($null -eq $sdkVersion) {
    exit 1
}

Write-Host "Found: $sdkVersion" -ForegroundColor Green
Write-Host ""

# Construct linker path
$linkerPath = Join-Path $vsPath "VC\Tools\MSVC\$vcVersion\bin\Hostx64\x64\link.exe"

if (-not (Test-Path $linkerPath)) {
    Write-Host "ERROR: Linker not found at expected path: $linkerPath" -ForegroundColor Red
    exit 1
}

Write-Host "Step 4: Creating Cargo configuration..." -ForegroundColor Yellow

# Create .cargo directory if it doesn't exist
$cargoDir = Join-Path $PSScriptRoot ".cargo"
if (-not (Test-Path $cargoDir)) {
    New-Item -ItemType Directory -Path $cargoDir | Out-Null
}

# Create config.toml with proper linker path
$configPath = Join-Path $cargoDir "config.toml"

# Escape backslashes for TOML
$escapedLinkerPath = $linkerPath -replace '\\', '\\'

$configContent = @"
[target.x86_64-pc-windows-msvc]
linker = "$escapedLinkerPath"

# Visual Studio paths detected automatically by fix-linker.ps1
# VS Install: $vsPath
# VC Tools: $vcVersion
# Windows SDK: $sdkVersion
"@

Set-Content -Path $configPath -Value $configContent -Encoding UTF8

Write-Host "Created: $configPath" -ForegroundColor Green
Write-Host ""

Write-Host "Step 5: Verifying configuration..." -ForegroundColor Yellow
Write-Host "Linker path: $linkerPath" -ForegroundColor White

if (Test-Path $linkerPath) {
    Write-Host "Linker verified successfully!" -ForegroundColor Green
} else {
    Write-Host "WARNING: Linker path verification failed." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=================================" -ForegroundColor Cyan
Write-Host "SUCCESS! Linker configured." -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "You can now run:" -ForegroundColor Yellow
Write-Host "  cargo build --release" -ForegroundColor White
Write-Host ""
Write-Host "If you still encounter issues, try:" -ForegroundColor Yellow
Write-Host "  1. Close and reopen your terminal" -ForegroundColor White
Write-Host "  2. Run: cargo clean" -ForegroundColor White
Write-Host "  3. Run: cargo build --release" -ForegroundColor White
Write-Host ""
