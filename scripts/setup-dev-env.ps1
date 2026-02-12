# MyMe Development Environment Setup Script
# Run from repo root: .\scripts\setup-dev-env.ps1

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
    Write-Host "[OK] Rust installed: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "[X] Rust not found. Please install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check CMake (PATH, then Qt/VS locations that scripts\build.ps1 uses)
Write-Host "Checking CMake installation..." -ForegroundColor Yellow
$cmakeExe = $null
if (Test-Command "cmake") {
    $cmakeExe = "cmake"
}
if (-not $cmakeExe) {
    $cmakePaths = @(
        "C:\Program Files\CMake\bin\cmake.exe",
        "C:\Qt\Tools\CMake_64\bin\cmake.exe"
    )
    # Add Visual Studio CMake (Community, Build Tools, etc.)
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $vsPath = (& $vsWhere -latest -property installationPath -ErrorAction SilentlyContinue) | Select-Object -First 1
        if ($vsPath -and ($vsPath.ToString().Trim() -ne "")) {
            $cmakePaths += Join-Path $vsPath "Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe"
        }
    }
    foreach ($path in $cmakePaths) {
        if (Test-Path $path) {
            $cmakeExe = $path
            break
        }
    }
}
if ($cmakeExe) {
    $cmakeVersion = & $cmakeExe --version 2>$null | Select-Object -First 1
    Write-Host "[OK] CMake installed: $cmakeVersion" -ForegroundColor Green
    if ($cmakeExe -ne "cmake") {
        Write-Host "  (found at: $cmakeExe - scripts\build.ps1 will use this)" -ForegroundColor Gray
    }
} else {
    Write-Host "[X] CMake not found. Please install from https://cmake.org/download/ or install Qt with CMake component." -ForegroundColor Red
    exit 1
}

# Check for Visual Studio
Write-Host "Checking Visual Studio..." -ForegroundColor Yellow
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath
    if ($vsPath) {
        Write-Host "[OK] Visual Studio found at: $vsPath" -ForegroundColor Green

        # Find the MSVC linker
        $linkExe = Get-ChildItem "$vsPath\VC\Tools\MSVC" -Recurse -Filter "link.exe" -ErrorAction SilentlyContinue |
                   Where-Object { $_.FullName -match "Hostx64\\x64" } |
                   Select-Object -First 1

        if ($linkExe) {
            Write-Host "[OK] MSVC linker found at: $($linkExe.DirectoryName)" -ForegroundColor Green
            Write-Host "  If you see 'cannot find link.exe' errors, run: .\scripts\fix-linker.ps1" -ForegroundColor Gray
        }
    }
} else {
    Write-Host "[X] Visual Studio not found. Please install Visual Studio with C++ workload" -ForegroundColor Red
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
        Write-Host "[OK] Qt found at: $path" -ForegroundColor Green
        $qtFound = $true
        # If path already has msvc kit, use it; else suggest first msvc subdir
        if ($path -match "msvc") {
            Write-Host "`nTo use this Qt installation, run: . .\scripts\setup-qt-env.ps1" -ForegroundColor Cyan
        } else {
            $qtCompiler = Get-ChildItem $path -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -match "msvc" } | Select-Object -First 1
            if ($qtCompiler) {
                $qtFullPath = Join-Path $path $qtCompiler.Name
                Write-Host "  Qt kit: $qtFullPath" -ForegroundColor Gray
                Write-Host "`nTo use this Qt installation, run: . .\scripts\setup-qt-env.ps1 -QtPath '$(Split-Path $path -Parent)'" -ForegroundColor Cyan
            }
        }
        break
    }
}

if (-not $qtFound) {
    Write-Host "[X] Qt not found. Please install Qt 6.x from https://www.qt.io/download" -ForegroundColor Red
    Write-Host "  Install to C:\Qt\6.10.1\msvc2022_64 or run . .\scripts\setup-qt-env.ps1 -QtPath 'C:\Qt'" -ForegroundColor Yellow
}

# scripts\build.ps1 creates build-qt when run; no need to pre-create
Write-Host "`nBuild: .\scripts\build.ps1 creates the build-qt directory and outputs myme-qt.exe" -ForegroundColor Gray

# Summary
Write-Host "`n====================================`n" -ForegroundColor Cyan
Write-Host "Setup Summary" -ForegroundColor Cyan
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. (Optional) Dot-source Qt env: . .\scripts\setup-qt-env.ps1" -ForegroundColor White
Write-Host "2. Full build (Rust + Qt): .\scripts\build.ps1" -ForegroundColor White
Write-Host "3. Run: .\build-qt\Release\myme-qt.exe" -ForegroundColor White
Write-Host "`nFor more help, see:" -ForegroundColor Yellow
Write-Host "- README.md - Project overview" -ForegroundColor White
Write-Host "- CLAUDE.md - Build commands and architecture" -ForegroundColor White
Write-Host "- WINDOWS_BUILD_FIX.md - Linker and Windows troubleshooting" -ForegroundColor White

Write-Host "`nSetup complete." -ForegroundColor Green
