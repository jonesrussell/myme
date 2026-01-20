# MyMe Development Environment Setup Script
# Run this in PowerShell to set up your development environment

Write-Host "MyMe Development Environment Setup" -ForegroundColor Cyan
Write-Host "====================================`n" -ForegroundColor Cyan

# Function to check if a command exists
function Test-Command($command) {
    try {
        if (Get-Command $command -ErrorAction Stop) {
            return $true
        }
    }
    catch {
        return $false
    }
}

# Check Rust
Write-Host "Checking Rust installation..." -ForegroundColor Yellow
if (Test-Command "cargo") {
    $rustVersion = cargo --version
    Write-Host "✓ Rust installed: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "✗ Rust not found. Please install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check CMake
Write-Host "Checking CMake installation..." -ForegroundColor Yellow
if (Test-Command "cmake") {
    $cmakeVersion = cmake --version | Select-Object -First 1
    Write-Host "✓ CMake installed: $cmakeVersion" -ForegroundColor Green
} else {
    Write-Host "✗ CMake not found. Please install from https://cmake.org/download/" -ForegroundColor Red
    exit 1
}

# Check for Visual Studio
Write-Host "Checking Visual Studio..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath
    if ($vsPath) {
        Write-Host "✓ Visual Studio found at: $vsPath" -ForegroundColor Green

        # Find the MSVC linker
        $linkExe = Get-ChildItem "$vsPath\VC\Tools\MSVC" -Recurse -Filter "link.exe" -ErrorAction SilentlyContinue |
                   Where-Object { $_.FullName -match "Hostx64\\x64" } |
                   Select-Object -First 1

        if ($linkExe) {
            Write-Host "✓ MSVC linker found at: $($linkExe.DirectoryName)" -ForegroundColor Green

            # Offer to update .cargo/config.toml
            Write-Host "`nWould you like to configure Cargo to use this linker? (Y/N)" -ForegroundColor Yellow
            $response = Read-Host

            if ($response -eq "Y" -or $response -eq "y") {
                $configPath = ".cargo\config.toml"
                $linkerPath = $linkExe.FullName -replace '\\', '\\'

                $config = @"
# Cargo configuration for MyMe project

# Windows-specific linker configuration
[target.x86_64-pc-windows-msvc]
linker = "$linkerPath"

# Optimization settings for development
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 2

# Release optimizations
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
"@

                $config | Out-File -FilePath $configPath -Encoding UTF8
                Write-Host "✓ Updated .cargo\config.toml with linker path" -ForegroundColor Green
            }
        }
    }
} else {
    Write-Host "✗ Visual Studio not found. Please install Visual Studio with C++ workload" -ForegroundColor Red
}

# Check Qt
Write-Host "`nChecking Qt installation..." -ForegroundColor Yellow
$qtPaths = @(
    "C:\Qt\6.7",
    "C:\Qt\6.6",
    "C:\Qt\6.5",
    "$env:USERPROFILE\Qt\6.7",
    "$env:USERPROFILE\Qt\6.6"
)

$qtFound = $false
foreach ($path in $qtPaths) {
    if (Test-Path $path) {
        Write-Host "✓ Qt found at: $path" -ForegroundColor Green
        $qtFound = $true

        # Suggest setting CMAKE_PREFIX_PATH
        $qtVersion = Split-Path -Leaf $path
        $qtCompiler = Get-ChildItem $path -Directory | Where-Object { $_.Name -match "msvc" } | Select-Object -First 1

        if ($qtCompiler) {
            $qtFullPath = Join-Path $path $qtCompiler.Name
            Write-Host "`nTo use this Qt installation, run:" -ForegroundColor Cyan
            Write-Host "`$env:CMAKE_PREFIX_PATH='$qtFullPath'" -ForegroundColor White
        }
        break
    }
}

if (-not $qtFound) {
    Write-Host "✗ Qt not found. Please install Qt 6.x from https://www.qt.io/download" -ForegroundColor Red
    Write-Host "  Install to one of these locations: $($qtPaths -join ', ')" -ForegroundColor Yellow
}

# Create build directory
Write-Host "`nCreating build directory..." -ForegroundColor Yellow
if (-not (Test-Path "build")) {
    New-Item -ItemType Directory -Path "build" | Out-Null
    Write-Host "✓ Created build directory" -ForegroundColor Green
} else {
    Write-Host "✓ Build directory already exists" -ForegroundColor Green
}

# Check for Golang todo API (optional)
Write-Host "`nChecking for Golang todo API (optional)..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/api/todos" -TimeoutSec 2 -ErrorAction Stop
    Write-Host "✓ Todo API is running on http://localhost:8080" -ForegroundColor Green
} catch {
    Write-Host "○ Todo API not running (optional for testing)" -ForegroundColor Gray
    Write-Host "  You can test the architecture without it" -ForegroundColor Gray
}

# Summary
Write-Host "`n====================================`n" -ForegroundColor Cyan
Write-Host "Setup Summary" -ForegroundColor Cyan
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. Open 'Developer Command Prompt for VS' from Start Menu" -ForegroundColor White
Write-Host "2. Navigate to this directory: cd $PWD" -ForegroundColor White
Write-Host "3. Run: cargo build" -ForegroundColor White
Write-Host "4. Run: cd build && cmake .. && cmake --build . --config Release" -ForegroundColor White
Write-Host "5. Run: .\build\Release\myme-qt.exe" -ForegroundColor White
Write-Host "`nFor more help, see:" -ForegroundColor Yellow
Write-Host "- QUICKSTART.md - Quick start guide" -ForegroundColor White
Write-Host "- BUILD.md - Detailed build instructions" -ForegroundColor White
Write-Host "- WINDOWS_BUILD_FIX.md - Windows-specific troubleshooting" -ForegroundColor White

Write-Host "`nSetup complete! 🚀" -ForegroundColor Green
