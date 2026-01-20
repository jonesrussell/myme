# Simple file check script
$files = @(
    "Cargo.toml",
    "src\main.rs",
    "crates\myme-core\Cargo.toml",
    "crates\myme-core\src\lib.rs",
    "crates\myme-core\src\app.rs",
    "crates\myme-core\src\config.rs",
    "crates\myme-core\src\plugin.rs",
    "crates\myme-services\Cargo.toml",
    "crates\myme-services\src\lib.rs",
    "crates\myme-services\src\todo.rs",
    "crates\myme-ui\Cargo.toml",
    "crates\myme-ui\build.rs",
    "crates\myme-ui\src\lib.rs",
    "crates\myme-ui\src\models\mod.rs",
    "crates\myme-ui\src\models\todo_model.rs",
    "crates\myme-ui\qml\Main.qml",
    "crates\myme-ui\qml\pages\TodoPage.qml",
    "qt-main\main.cpp",
    "CMakeLists.txt",
    "README.md",
    "BUILD.md"
)

Write-Host "Checking MyMe architecture files..." -ForegroundColor Cyan
Write-Host ""

$missing = 0
foreach ($file in $files) {
    if (Test-Path $file) {
        Write-Host "OK: $file" -ForegroundColor Green
    } else {
        Write-Host "MISSING: $file" -ForegroundColor Red
        $missing++
    }
}

Write-Host ""
if ($missing -eq 0) {
    Write-Host "All files present! Architecture is complete." -ForegroundColor Green
} else {
    Write-Host "$missing files missing." -ForegroundColor Red
}
