# Set Qt environment variables for cargo build (cxx-qt needs these before build starts)
# Usage: . .\set-qt-env.ps1    (dot-source, then run cargo build)
#    or: .\set-qt-env.ps1; cargo build --release -p myme-ui

$qtPath = if ($env:QT_PATH) { $env:QT_PATH } else { "C:\Qt\6.10.1\msvc2022_64" }
$env:CMAKE_PREFIX_PATH = $qtPath
$env:Qt6_DIR = "$qtPath\lib\cmake\Qt6"
$env:QMAKE = "$qtPath\bin\qmake.exe"
Write-Host "Qt environment set: $qtPath" -ForegroundColor Green
