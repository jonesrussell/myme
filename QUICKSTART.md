# Quick Start Guide

## Getting Started with MyMe

This guide will help you get the MyMe application up and running.

## Prerequisites Checklist

- [ ] Rust installed (via rustup)
- [ ] Qt 6.x installed
- [ ] CMake installed
- [ ] C++ compiler (Visual Studio on Windows)
- [ ] Golang todo API running (optional for testing)

## 1. Fix Windows Linker (Windows Only)

If you're on Windows, you need to fix the linker PATH issue first.

**Easiest Solution**: Open **"Developer Command Prompt for VS"** from Start Menu

Then navigate to your project:
```bash
cd C:\Users\jones\dev\myme
```

**Alternative**: Create `.cargo/config.toml` with correct linker path (see [WINDOWS_BUILD_FIX.md](WINDOWS_BUILD_FIX.md))

## 2. Build Rust Crates

```bash
# From project root
cargo build --release
```

This will:
- Compile all workspace crates
- Generate cxx-qt bridge code
- Create `libmyme_ui.a` in `target/release/`

**Expected output**: "Finished release [optimized] target(s)"

## 3. Build Qt Application

```bash
# Create build directory
mkdir build
cd build

# Configure with CMake
cmake ..

# Build
cmake --build . --config Release
```

**Expected output**: `myme-qt.exe` in `build/Release/`

## 4. Configure Todo API

Edit the config file that will be created on first run:

**Windows**: `%APPDATA%\myme\config.toml`

```toml
[services]
todo_api_url = "http://localhost:8080"
```

Or set a different URL if your Golang todo API is running elsewhere.

## 5. Run the Application

### Option A: Run Rust Binary (Architecture Test)

```bash
cargo run
```

This will:
- Initialize the application
- Load configuration
- Display config info
- Exit gracefully

**Expected output**:
```
MyMe - Personal Productivity & Dev Hub
Architecture initialized successfully!

Configuration:
  Todo API: http://localhost:8080
  Config directory: C:\Users\jones\AppData\Roaming\myme
```

### Option B: Run Qt Application (Full UI)

```bash
# From project root
.\build\Release\myme-qt.exe
```

This will:
- Launch the Qt/QML application
- Show the main window with drawer navigation
- Allow you to interact with todos (if API is running)

## 6. Test with Todo API

If you have the Golang todo API running:

1. Start your Golang todo API server
2. Ensure it's running on the configured port (default: 8080)
3. Launch the Qt application
4. Click "Todos" in the drawer
5. Click the refresh button
6. You should see todos from your API

## Troubleshooting

### "Cannot find link.exe" or linker errors

→ See [WINDOWS_BUILD_FIX.md](WINDOWS_BUILD_FIX.md)

### "Qt6 not found"

Set CMAKE_PREFIX_PATH:
```bash
set CMAKE_PREFIX_PATH=C:\Qt\6.x\msvc2019_64
cmake ..
```

### "cxx-qt headers not found"

Ensure you've run `cargo build` first to generate the bridge code.

### QML files not loading

Check that QML import paths are correct in CMakeLists.txt.

### "Failed to connect to API"

- Verify Golang todo API is running
- Check the URL in config.toml
- Check firewall settings

## Development Workflow

### Making Rust Changes

1. Edit Rust source files
2. Run `cargo build`
3. Rebuild Qt app: `cmake --build build`
4. Run `.\build\Release\myme-qt.exe`

### Making QML Changes

1. Edit QML files
2. No rebuild needed!
3. Just restart the application

### Adding New Dependencies

1. Add to workspace dependencies in root `Cargo.toml`
2. Add to specific crate's `Cargo.toml`
3. Run `cargo build`

## Useful Commands

```bash
# Check all crates without building
cargo check --workspace

# Run tests
cargo test --workspace

# Clean build artifacts
cargo clean
rm -rf build

# View logs with debug info
$env:RUST_LOG="debug"
cargo run

# Qt debug logging
$env:QT_LOGGING_RULES="*.debug=true"
.\build\Release\myme-qt.exe
```

## Next Steps

Once you have the application running:

1. Test all todo operations (add, complete, delete)
2. Verify configuration persistence
3. Check logs for any warnings
4. Review the architecture documentation
5. Start planning Phase 2 features

## Getting Help

- **Build Issues**: Check [BUILD.md](BUILD.md)
- **Windows Linker**: Check [WINDOWS_BUILD_FIX.md](WINDOWS_BUILD_FIX.md)
- **Architecture**: Check [ARCHITECTURE_SUMMARY.md](ARCHITECTURE_SUMMARY.md)
- **Status**: Check [PROJECT_STATUS.md](PROJECT_STATUS.md)

## Success Indicators

✅ `cargo build` completes without errors
✅ `cmake --build build` completes successfully
✅ Application launches and shows main window
✅ Can navigate to Todos page
✅ Configuration file is created
✅ Logs show "MyMe application started"

You're ready to go! 🚀
