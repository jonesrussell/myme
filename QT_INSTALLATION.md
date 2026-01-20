# Qt 6 Installation for MyMe

## Issue

The build is failing with `Could not find Qt installation: QtMissing`. cxx-qt requires Qt 6 to be installed and properly configured.

## Installation Methods

### Option 1: Qt Online Installer (Recommended)

1. **Download Qt Installer**
   - Go to: https://www.qt.io/download-qt-installer
   - Download the Qt Online Installer for Windows

2. **Run Installer**
   - Launch the installer
   - Create/login to Qt account (free for open source)
   - Select "Custom Installation"

3. **Select Components** (Minimum required):
   - ✅ Qt 6.7 or later (latest stable version)
   - ✅ Qt 6.x → Desktop gcc 64-bit (for Windows MSVC)
   - ✅ Qt 6.x → Qt 5 Compatibility Module
   - ✅ Developer and Designer Tools → CMake
   - Optional: Qt Creator IDE

4. **Installation Location**
   - Default: `C:\Qt\6.x.x`
   - Note this path for configuration

### Option 2: vcpkg (Package Manager)

```powershell
# Install vcpkg if not already installed
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg
cd C:\vcpkg
.\bootstrap-vcpkg.bat

# Install Qt6
.\vcpkg install qt6-base:x64-windows
.\vcpkg integrate install
```

### Option 3: Chocolatey

```powershell
choco install qt6 -y
```

## Configuration

After installation, you need to tell Rust where to find Qt:

### Set Environment Variables

**PowerShell (Current Session):**
```powershell
$env:CMAKE_PREFIX_PATH = "C:\Qt\6.7.0\msvc2019_64"
$env:Qt6_DIR = "C:\Qt\6.7.0\msvc2019_64\lib\cmake\Qt6"
```

**PowerShell (Permanent):**
```powershell
[System.Environment]::SetEnvironmentVariable("CMAKE_PREFIX_PATH", "C:\Qt\6.7.0\msvc2019_64", "User")
[System.Environment]::SetEnvironmentVariable("Qt6_DIR", "C:\Qt\6.7.0\msvc2019_64\lib\cmake\Qt6", "User")
```

**Or via System Properties:**
1. Press Windows + X → System
2. Advanced system settings → Environment Variables
3. Add to User variables:
   - Variable: `CMAKE_PREFIX_PATH`
   - Value: `C:\Qt\6.7.0\msvc2019_64` (adjust version)
   - Variable: `Qt6_DIR`
   - Value: `C:\Qt\6.7.0\msvc2019_64\lib\cmake\Qt6`

### Verify Qt Installation

```powershell
# Check if qmake is accessible
qmake --version

# If not, add to PATH:
$env:PATH = "C:\Qt\6.7.0\msvc2019_64\bin;$env:PATH"
```

## Quick Setup Script

Save this as `setup-qt-env.ps1`:

```powershell
# Qt Environment Setup for MyMe
param(
    [string]$QtPath = "C:\Qt"
)

Write-Host "Setting up Qt environment for MyMe..." -ForegroundColor Cyan

# Find Qt installation
$qtVersions = Get-ChildItem $QtPath -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '^\d+\.\d+\.\d+$' } |
    Sort-Object Name -Descending

if ($qtVersions.Count -eq 0) {
    Write-Host "ERROR: No Qt installation found in $QtPath" -ForegroundColor Red
    Write-Host "Please install Qt 6.x first. See QT_INSTALLATION.md" -ForegroundColor Yellow
    exit 1
}

$latestQt = $qtVersions[0]
$qtVersionPath = Join-Path $QtPath $latestQt.Name

# Find MSVC compiler version
$msvcDirs = Get-ChildItem $qtVersionPath -Directory | Where-Object { $_.Name -match '^msvc\d+_64$' }

if ($msvcDirs.Count -eq 0) {
    Write-Host "ERROR: No MSVC 64-bit build found in Qt installation" -ForegroundColor Red
    exit 1
}

$qtCompilerPath = $msvcDirs[0].FullName
$qtCmakePath = Join-Path $qtCompilerPath "lib\cmake\Qt6"

Write-Host "Found Qt $($latestQt.Name)" -ForegroundColor Green
Write-Host "Path: $qtCompilerPath" -ForegroundColor White

# Set environment variables for current session
$env:CMAKE_PREFIX_PATH = $qtCompilerPath
$env:Qt6_DIR = $qtCmakePath
$env:PATH = "$qtCompilerPath\bin;$env:PATH"

Write-Host ""
Write-Host "Qt environment configured for current session!" -ForegroundColor Green
Write-Host ""
Write-Host "To make permanent, run:" -ForegroundColor Yellow
Write-Host "  [System.Environment]::SetEnvironmentVariable('CMAKE_PREFIX_PATH', '$qtCompilerPath', 'User')" -ForegroundColor White
Write-Host "  [System.Environment]::SetEnvironmentVariable('Qt6_DIR', '$qtCmakePath', 'User')" -ForegroundColor White
Write-Host ""
Write-Host "You can now run: cargo build --release" -ForegroundColor Cyan
```

## Common Installation Paths

Qt installations are typically in:
- **Qt Installer:** `C:\Qt\6.x.x\msvc2019_64`
- **vcpkg:** `C:\vcpkg\installed\x64-windows`
- **Chocolatey:** `C:\ProgramData\chocolatey\lib\qt6`

## Building After Qt Installation

1. **Set up environment** (if not permanent):
   ```powershell
   .\setup-qt-env.ps1
   ```

2. **Build MyMe**:
   ```powershell
   .\build.ps1
   # or
   cargo build --release
   ```

## Verification

After setting up Qt, verify with:

```powershell
# Check environment variables
echo $env:CMAKE_PREFIX_PATH
echo $env:Qt6_DIR

# Check qmake
qmake --version

# Should output something like:
# QMake version 3.1
# Using Qt version 6.7.0 in C:\Qt\6.7.0\msvc2019_64\lib
```

## Troubleshooting

### "Could not find Qt installation"

1. Verify Qt is installed: `qmake --version`
2. Check environment variables are set correctly
3. Restart your terminal/IDE after setting variables
4. Ensure you installed the MSVC 64-bit components

### "Qt version too old"

cxx-qt requires Qt 6.2 or later. Update your Qt installation if needed.

### CMake errors

Ensure CMake is installed:
```powershell
cmake --version
# If missing, install via Qt installer or: choco install cmake
```

## Next Steps

After Qt is installed and configured:

1. Restart your terminal/PowerShell
2. Run the build script: `.\build.ps1`
3. Continue with Phase 2 development

## Current Status

- ✅ Rust workspace structure complete
- ✅ Phase 1 code (Godo integration) complete
- ✅ Phase 2 auth framework complete
- ⏳ **Pending: Qt 6 installation for UI compilation**
- ⏳ Pending: Complete Phase 2 GitHub integration
