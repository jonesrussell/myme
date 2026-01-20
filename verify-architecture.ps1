# Architecture Verification Script
# This script verifies that all expected files are in place

Write-Host "MyMe Architecture Verification" -ForegroundColor Cyan
Write-Host "==============================`n" -ForegroundColor Cyan

$errors = 0
$warnings = 0

# Function to check file exists
function Test-File($path, $description) {
    if (Test-Path $path) {
        Write-Host "✓ $description" -ForegroundColor Green
        return $true
    } else {
        Write-Host "✗ $description - NOT FOUND: $path" -ForegroundColor Red
        $script:errors++
        return $false
    }
}

# Check workspace structure
Write-Host "Checking workspace structure..." -ForegroundColor Yellow
Test-File "Cargo.toml" "Workspace manifest"
Test-File "src\main.rs" "Binary entry point"

Write-Host "`nChecking core crate..." -ForegroundColor Yellow
Test-File "crates\myme-core\Cargo.toml" "Core manifest"
Test-File "crates\myme-core\src\lib.rs" "Core library"
Test-File "crates\myme-core\src\app.rs" "Application logic"
Test-File "crates\myme-core\src\config.rs" "Configuration management"
Test-File "crates\myme-core\src\plugin.rs" "Plugin system"

Write-Host "`nChecking services crate..." -ForegroundColor Yellow
Test-File "crates\myme-services\Cargo.toml" "Services manifest"
Test-File "crates\myme-services\src\lib.rs" "Services library"
Test-File "crates\myme-services\src\todo.rs" "Todo API client"

Write-Host "`nChecking UI crate..." -ForegroundColor Yellow
Test-File "crates\myme-ui\Cargo.toml" "UI manifest"
Test-File "crates\myme-ui\build.rs" "cxx-qt build script"
Test-File "crates\myme-ui\src\lib.rs" "UI library"
Test-File "crates\myme-ui\src\models\mod.rs" "Models module"
Test-File "crates\myme-ui\src\models\todo_model.rs" "TodoModel bridge"

Write-Host "`nChecking QML files..." -ForegroundColor Yellow
Test-File "crates\myme-ui\qml\Main.qml" "Main window"
Test-File "crates\myme-ui\qml\pages\TodoPage.qml" "Todo page"

Write-Host "`nChecking Qt/C++ integration..." -ForegroundColor Yellow
Test-File "qt-main\main.cpp" "Qt main entry point"
Test-File "CMakeLists.txt" "CMake build config"

Write-Host "`nChecking documentation..." -ForegroundColor Yellow
Test-File "README.md" "Project overview"
Test-File "BUILD.md" "Build instructions"
Test-File "QUICKSTART.md" "Quick start guide"
Test-File "DEVELOPMENT.md" "Development guide"
Test-File "ARCHITECTURE_SUMMARY.md" "Architecture summary"
Test-File "ARCHITECTURE_DIAGRAM.md" "Architecture diagram"
Test-File "PROJECT_STATUS.md" "Project status"
Test-File "WINDOWS_BUILD_FIX.md" "Windows troubleshooting"

Write-Host "`nChecking configuration files..." -ForegroundColor Yellow
Test-File ".gitignore" "Git ignore file"
Test-File ".cargo\config.toml" "Cargo configuration"

# Verify file structure
Write-Host "`nVerifying crate structure..." -ForegroundColor Yellow

$expectedDirs = @(
    "crates",
    "crates\myme-core",
    "crates\myme-core\src",
    "crates\myme-services",
    "crates\myme-services\src",
    "crates\myme-ui",
    "crates\myme-ui\src",
    "crates\myme-ui\src\models",
    "crates\myme-ui\qml",
    "crates\myme-ui\qml\pages",
    "qt-main",
    ".cargo"
)

foreach ($dir in $expectedDirs) {
    if (Test-Path $dir -PathType Container) {
        Write-Host "✓ Directory exists: $dir" -ForegroundColor Green
    } else {
        Write-Host "✗ Directory missing: $dir" -ForegroundColor Red
        $errors++
    }
}

# Count lines of code
Write-Host "`nCode statistics..." -ForegroundColor Yellow

$rustFiles = Get-ChildItem -Path "." -Include "*.rs" -Recurse -File |
             Where-Object { $_.FullName -notmatch "\\target\\" }

$qmlFiles = Get-ChildItem -Path "." -Include "*.qml" -Recurse -File

$rustLines = ($rustFiles | Get-Content | Measure-Object -Line).Lines
$qmlLines = ($qmlFiles | Get-Content | Measure-Object -Line).Lines

Write-Host "Rust files: $($rustFiles.Count) ($rustLines lines)" -ForegroundColor Cyan
Write-Host "QML files: $($qmlFiles.Count) ($qmlLines lines)" -ForegroundColor Cyan
Write-Host "Total code: $($rustLines + $qmlLines) lines" -ForegroundColor Cyan

# Check Cargo.toml content
Write-Host "`nVerifying workspace configuration..." -ForegroundColor Yellow
$cargoContent = Get-Content "Cargo.toml" -Raw

if ($cargoContent -match "\[workspace\]") {
    Write-Host "✓ Workspace configuration found" -ForegroundColor Green
} else {
    Write-Host "✗ Workspace configuration missing" -ForegroundColor Red
    $errors++
}

if ($cargoContent -match "myme-core") {
    Write-Host "✓ myme-core crate referenced" -ForegroundColor Green
} else {
    Write-Host "✗ myme-core crate not referenced" -ForegroundColor Red
    $errors++
}

if ($cargoContent -match "myme-ui") {
    Write-Host "✓ myme-ui crate referenced" -ForegroundColor Green
} else {
    Write-Host "✗ myme-ui crate not referenced" -ForegroundColor Red
    $errors++
}

if ($cargoContent -match "myme-services") {
    Write-Host "✓ myme-services crate referenced" -ForegroundColor Green
} else {
    Write-Host "✗ myme-services crate not referenced" -ForegroundColor Red
    $errors++
}

# Summary
Write-Host "`n==============================" -ForegroundColor Cyan
Write-Host "Verification Summary" -ForegroundColor Cyan
Write-Host "==============================`n" -ForegroundColor Cyan

if ($errors -eq 0 -and $warnings -eq 0) {
    Write-Host "All checks passed! Architecture is complete." -ForegroundColor Green
    Write-Host ""
    Write-Host "The project structure is correct and all files are in place." -ForegroundColor White
    Write-Host "Ready to proceed with building!" -ForegroundColor White
    exit 0
} elseif ($errors -eq 0) {
    Write-Host "$warnings warnings found" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Architecture is mostly complete with minor issues." -ForegroundColor White
    exit 0
} else {
    Write-Host "$errors errors found" -ForegroundColor Red
    if ($warnings -gt 0) {
        Write-Host "$warnings warnings found" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "Please fix the errors before proceeding." -ForegroundColor White
    exit 1
}
