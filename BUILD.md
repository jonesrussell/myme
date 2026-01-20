# Build Instructions

This document describes how to build the MyMe application with full Qt/QML support.

## Prerequisites

### 1. Rust

Install Rust from [rustup.rs](https://rustup.rs/)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Qt 6

Install Qt 6.x with the following components:
- Qt 6 Core
- Qt 6 Quick
- Qt 6 QuickControls2
- Kirigami (if not bundled)

**Windows**: Download from [qt.io](https://www.qt.io/download)

**macOS** (via Homebrew):
```bash
brew install qt@6
```

**Linux** (Ubuntu/Debian):
```bash
sudo apt install qt6-base-dev qt6-declarative-dev qml6-module-org-kde-kirigami2
```

### 3. CMake

**Windows**: Download from [cmake.org](https://cmake.org/download/)

**macOS**:
```bash
brew install cmake
```

**Linux**:
```bash
sudo apt install cmake
```

### 4. C++ Compiler

- **Windows**: Visual Studio 2019 or later with C++ workload
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Linux**: GCC or Clang (`sudo apt install build-essential`)

## Build Steps

### 1. Build Rust Crates

First, build all Rust crates including the cxx-qt bridge:

```bash
cargo build --release
```

This will:
- Compile all workspace crates
- Generate cxx-qt bridge code in `target/cxxqt/`
- Create static library `libmyme_ui.a` (or `.lib` on Windows)

### 2. Build Qt Application

Configure and build with CMake:

```bash
mkdir build
cd build
cmake ..
cmake --build . --config Release
```

### 3. Run

After building:

**Linux/macOS**:
```bash
./build/myme-qt
```

**Windows**:
```bash
.\build\Release\myme-qt.exe
```

## Development Workflow

### Iterative Development

When making changes to Rust code:

```bash
# 1. Rebuild Rust crates
cargo build

# 2. Rebuild Qt application (from build/ directory)
cmake --build . --config Debug
```

When making changes to QML only (no rebuild needed):
- QML files are loaded at runtime
- Just restart the application

### Debugging

**Rust side**:
```bash
RUST_LOG=debug cargo run
```

**Qt side**:
```bash
QT_LOGGING_RULES="*.debug=true" ./build/myme-qt
```

## Troubleshooting

### "cxx-qt headers not found"

Ensure you've run `cargo build` first. The cxx-qt code generation happens during the Rust build.

### "Qt libraries not found"

Set Qt installation path:

```bash
export CMAKE_PREFIX_PATH="/path/to/Qt/6.x/gcc_64"
cmake ..
```

### "libmyme_ui.a not found"

Build the Rust crates first:
```bash
cargo build --release
```

### Kirigami components not loading

Ensure QML import paths are set correctly. Check that Kirigami is installed:

**Linux**:
```bash
sudo apt install qml6-module-org-kde-kirigami2
```

## Current Limitations (Phase 1)

1. **TodoModel initialization**: The TodoModel needs to be initialized with a TodoClient and tokio runtime. This requires additional C++ code to call Rust initialization functions.

2. **Async updates**: The current implementation spawns async tasks but doesn't properly signal the QML layer when data changes. Phase 1 demonstrates the architecture; full functionality requires Qt signals integration.

3. **Resource bundling**: QML files are currently loaded from filesystem. Production builds should use Qt resource system (qrc files).

## Next Steps for Full Integration

1. Create Rust initialization function callable from C++
2. Implement Qt signal emissions from Rust async tasks
3. Add proper error handling in UI layer
4. Bundle QML resources in qrc file
5. Add application icon and metadata

These will be addressed in the completion of Phase 1.
