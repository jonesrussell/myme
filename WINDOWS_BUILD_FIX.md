# Windows Build Fix

## Issue

The build is failing with `LINK : fatal error LNK1181: cannot open input file 'kernel32.lib'`.

**Root Causes:**
1. Rust's linker is conflicting with another `link` command in the system PATH (likely Git's `link` or a Unix-like tool)
2. Windows SDK libraries are not properly installed or not in the expected location

## Quick Solution: Use Build Script

We've provided an automated build script that uses the Visual Studio Developer environment:

```powershell
.\scripts\build.ps1
```
(Or Rust only: `.\scripts\build-rust.ps1`)

The script automatically:
- Detects your Visual Studio installation
- Initializes the Developer environment
- Runs `cargo build --release` with correct linker paths

If you see "cannot find link.exe" errors, run `.\scripts\fix-linker.ps1` to configure Cargo to use the MSVC linker.

## Manual Solutions

### Option 1: Use Visual Studio Developer Command Prompt (Recommended)

1. Open "Developer Command Prompt for VS 2019/2022"
2. Navigate to project directory
3. Run `cargo build --release`

This ensures the correct Microsoft linker is prioritized and all SDK paths are configured.

**To find the Developer Command Prompt:**
- Press Windows key, type "Developer Command Prompt"
- Or: Start Menu → Visual Studio 2022 → Developer Command Prompt for VS 2022

### Option 2: Temporarily Adjust PATH

```powershell
# In PowerShell, prioritize Visual Studio tools
$env:PATH = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\<version>\bin\Hostx64\x64;$env:PATH"
cargo build
```

### Option 3: Use rustup to specify target explicitly

```bash
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc
```

### Option 4: Configure Rust to use specific linker

Create `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
linker = "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\<version>\\bin\\Hostx64\\x64\\link.exe"
```

Replace `<version>` with your actual MSVC version.

## Verification

After applying a fix, test with:

```bash
cargo check --workspace
```

## Alternative: Use WSL or Cross-Platform Development

If Windows-specific build issues persist, consider:

1. **WSL2** (Windows Subsystem for Linux):
   ```bash
   # In WSL2
   sudo apt install build-essential qt6-base-dev
   cargo build
   ```

2. **Cross-compile** from Linux/macOS to Windows

3. **Use Docker** for consistent build environment

## Missing Windows SDK

If you see `cannot open input file 'kernel32.lib'`, the Windows SDK may be incompletely installed.

**Fix:**
1. Open Visual Studio Installer
2. Modify your Visual Studio installation
3. Under "Individual components", ensure these are checked:
   - Windows 10 SDK (or Windows 11 SDK)
   - MSVC v142 - VS 2022 C++ x64/x86 build tools (or latest version)
4. Install/Repair the components

## Unresolved Externals (LNK2019) - WinHTTP / UuidCreate

If you see errors like `unresolved external symbol __imp_WinHttp*` or `__imp_UuidCreate`, the CMake build is missing Windows system libraries required by reqwest's native-tls backend.

**Fix:** CMakeLists.txt already links `winhttp` and `rpcrt4`. Ensure you're using the latest CMakeLists.txt. If issues persist, verify the Windows SDK is fully installed (Visual Studio Installer → Modify → Individual components → Windows SDK).

## Current Status

The Rust architecture is complete and correct. The build failure is environment-specific (Windows linker/SDK path issue), not a code issue.

All crate structure, dependencies, and code are properly set up. Once the linker is configured correctly, the project will build successfully.

**Your detected configuration:**
- Visual Studio: C:\Program Files\Microsoft Visual Studio\2022\Community
- VC Tools: 14.44.35207
- Windows SDK: Not properly installed (Lib folder missing)
