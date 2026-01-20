# Build Qt application for MyMe

Write-Host "MyMe - Qt Application Build Script" -ForegroundColor Cyan
Write-Host "===================================`n"

# First, build Rust library
Write-Host "Step 1: Building Rust library..." -ForegroundColor Yellow
& powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\build.ps1"

if ($LASTEXITCODE -ne 0) {
    Write-Host "`nRust build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "`nStep 2: Finding CMake..." -ForegroundColor Yellow

# Find CMake
$cmakePaths = @(
    "C:\Program Files\CMake\bin\cmake.exe",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\CMake\bin\cmake.exe",
    "C:\Qt\Tools\CMake_64\bin\cmake.exe"
)

$cmake = $null
foreach ($path in $cmakePaths) {
    if (Test-Path $path) {
        $cmake = $path
        Write-Host "Found CMake: $cmake" -ForegroundColor Green
        break
    }
}

if (-not $cmake) {
    Write-Host "ERROR: CMake not found! Please install CMake." -ForegroundColor Red
    exit 1
}

Write-Host "`nStep 3: Configuring CMake..." -ForegroundColor Yellow

# Set Qt path
$qtPath = "C:\Qt\6.10.1\msvc2022_64"
$env:CMAKE_PREFIX_PATH = $qtPath
$env:Qt6_DIR = "$qtPath\lib\cmake\Qt6"

# Create build directory
$buildDir = "$PSScriptRoot\build-qt"
if (-not (Test-Path $buildDir)) {
    New-Item -ItemType Directory -Path $buildDir | Out-Null
}

# Configure CMake
Push-Location $buildDir
& $cmake -G "Visual Studio 17 2022" -A x64 `
    -DCMAKE_BUILD_TYPE=Release `
    -DCMAKE_PREFIX_PATH="$qtPath" `
    -DQt6_DIR="$qtPath\lib\cmake\Qt6" `
    ..

if ($LASTEXITCODE -ne 0) {
    Write-Host "`nCMake configuration failed!" -ForegroundColor Red
    Pop-Location
    exit 1
}

Write-Host "`nStep 4: Building Qt application..." -ForegroundColor Yellow

& $cmake --build . --config Release

Pop-Location

if ($LASTEXITCODE -ne 0) {
    Write-Host "`n==================================" -ForegroundColor Cyan
    Write-Host "QT BUILD FAILED" -ForegroundColor Red
    Write-Host "==================================" -ForegroundColor Cyan
    exit 1
}

Write-Host "`nStep 5: Deploying Qt dependencies..." -ForegroundColor Yellow

# Deploy Qt DLLs and plugins
$windeployqt = "$qtPath\bin\windeployqt.exe"
$exePath = "$buildDir\Release\myme-qt.exe"
$qmlDir = "$PSScriptRoot\crates\myme-ui\qml"

if (-not (Test-Path $windeployqt)) {
    Write-Host "WARNING: windeployqt not found at $windeployqt" -ForegroundColor Yellow
    Write-Host "Qt dependencies will need to be deployed manually." -ForegroundColor Yellow
} else {
    & $windeployqt $exePath --qmldir $qmlDir --release --no-translations

    if ($LASTEXITCODE -eq 0) {
        Write-Host "Qt dependencies deployed successfully." -ForegroundColor Green
    } else {
        Write-Host "WARNING: windeployqt failed. Application may not run standalone." -ForegroundColor Yellow
    }
}

Write-Host "`n==================================" -ForegroundColor Cyan
Write-Host "BUILD COMPLETE!" -ForegroundColor Green
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "`nExecutable location: $buildDir\Release\myme-qt.exe" -ForegroundColor Green
Write-Host "You can now run the application directly.`n" -ForegroundColor Green
