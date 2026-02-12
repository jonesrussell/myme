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
            Write-Host "  If you see 'cannot find link.exe' errors, run: .\fix-linker.ps1" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "✗ Visual Studio not found. Please install Visual Studio with C++ workload" -ForegroundColor Red
}

# Check Qt
Write-Host "`nChecking Qt installation..." -ForegroundColor Yellow
$qtPaths = @(
    "C:\Qt\6.10.1\msvc2022_64",
    "C:\Qt\6.7",
    "C:\Qt\6.6",
    "$env:USERPROFILE\Qt\6.7",
    "$env:USERPROFILE\Qt\6.6"
)

$qtFound = $false
foreach ($path in $qtPaths) {
    if (Test-Path $path) {
        Write-Host "✓ Qt found at: $path" -ForegroundColor Green
        $qtFound = $true
        # If path already has msvc kit, use it; else suggest first msvc subdir
        if ($path -match "msvc") {
            Write-Host "`nTo use this Qt installation, run: .\setup-qt-env.ps1" -ForegroundColor Cyan
        } else {
            $qtCompiler = Get-ChildItem $path -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match "msvc" } | Select-Object -First 1
            if ($qtCompiler) {
                $qtFullPath = Join-Path $path $qtCompiler.Name
                Write-Host "`nTo use this Qt installation, run: .\setup-qt-env.ps1 -QtPath '$(Split-Path $path -Parent)'" -ForegroundColor Cyan
            }
        }
        break
    }
}

if (-not $qtFound) {
    Write-Host "✗ Qt not found. Please install Qt 6.x from https://www.qt.io/download" -ForegroundColor Red
    Write-Host "  Install to C:\Qt\6.10.1\msvc2022_64 or run .\setup-qt-env.ps1 -QtPath 'C:\Qt'" -ForegroundColor Yellow
}

# build-qt.ps1 creates build-qt when run; no need to pre-create
Write-Host "`nBuild: .\build-qt.ps1 creates the build-qt directory and outputs myme-qt.exe" -ForegroundColor Gray

# Summary
Write-Host "`n====================================`n" -ForegroundColor Cyan
Write-Host "Setup Summary" -ForegroundColor Cyan
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. (Optional) Dot-source Qt env: . .\setup-qt-env.ps1" -ForegroundColor White
Write-Host "2. Full build (Rust + Qt): .\build-qt.ps1" -ForegroundColor White
Write-Host "3. Run: .\build-qt\Release\myme-qt.exe" -ForegroundColor White
Write-Host "`nFor more help, see:" -ForegroundColor Yellow
Write-Host "- README.md - Project overview" -ForegroundColor White
Write-Host "- CLAUDE.md - Build commands and architecture" -ForegroundColor White
Write-Host "- WINDOWS_BUILD_FIX.md - Linker and Windows troubleshooting" -ForegroundColor White

Write-Host "`nSetup complete!" -ForegroundColor Green
