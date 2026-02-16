#!/usr/bin/env bash
# Build Qt application for MyMe on Linux/WSL2
# Run from repo root: ./scripts/build.sh
#
# Prerequisites:
#   - Rust, CMake, Qt 6.x (apt: qt6-base-dev qt6-declarative-dev, or aqtinstall)
#   - libsecret (for OAuth keyring): libsecret-1-dev
#   - WSL2 GUI: requires WSLg (default in recent WSL2)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build"
QML_DIR="$PROJECT_ROOT/crates/myme-ui/qml"

echo "MyMe - Qt Application Build Script (Linux/WSL2)"
echo "==============================================="
echo ""

# Detect Qt path: QT_PATH or QT_ROOT_DIR (from install-qt-action) or auto-detect
if [ -n "$QT_PATH" ]; then
    QT_ROOT="$QT_PATH"
elif [ -n "$QT_ROOT_DIR" ]; then
    QT_ROOT="$QT_ROOT_DIR"
elif [ -d "$PROJECT_ROOT/qt-install" ]; then
    # aqtinstall layout: qt-install/version/arch
    QT_CANDIDATE=$(find "$PROJECT_ROOT/qt-install" -name "qmake" -path "*/bin/qmake" 2>/dev/null | head -1)
    if [ -n "$QT_CANDIDATE" ]; then
        QT_ROOT="$(cd "$(dirname "$(dirname "$QT_CANDIDATE")")" && pwd)"
    fi
fi

if [ -z "$QT_ROOT" ]; then
    # Try system Qt via pkg-config or default paths
    if command -v pkg-config &>/dev/null && pkg-config --exists Qt6Core 2>/dev/null; then
        QT_ROOT="$(pkg-config --variable=prefix Qt6Core)"
    fi
    if [ -z "$QT_ROOT" ] && [ -d "/usr/lib/x86_64-linux-gnu/cmake/Qt6" ]; then
        QT_ROOT="/usr"
    fi
    if [ -z "$QT_ROOT" ]; then
        echo "ERROR: Qt 6 not found. Install via:"
        echo "  sudo apt install qt6-base-dev qt6-declarative-dev libxcb-cursor0"
        echo "Or use aqtinstall and set QT_PATH to the Qt root (e.g. qt-install/6.10.1/gcc_64)"
        exit 1
    fi
fi

export CMAKE_PREFIX_PATH="$QT_ROOT"
# Debian/Ubuntu may use lib/x86_64-linux-gnu for cmake configs
if [ -d "$QT_ROOT/lib/x86_64-linux-gnu/cmake/Qt6" ]; then
    export Qt6_DIR="$QT_ROOT/lib/x86_64-linux-gnu/cmake/Qt6"
else
    export Qt6_DIR="$QT_ROOT/lib/cmake/Qt6"
fi
if [ -x "$QT_ROOT/bin/qmake" ]; then
    export QMAKE="$QT_ROOT/bin/qmake"
elif command -v qmake6 &>/dev/null; then
    export QMAKE="qmake6"
else
    export QMAKE="qmake"
fi
echo "Using Qt from: $QT_ROOT"
echo ""

# Step 1: Build Rust library
echo "Step 1: Building Rust library..."
cd "$PROJECT_ROOT"
cargo build -p myme-ui --release
if [ $? -ne 0 ]; then
    echo "Rust build failed!"
    exit 1
fi
echo ""

# Step 2: Configure CMake
echo "Step 2: Configuring CMake..."
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"
cmake "$PROJECT_ROOT" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_PREFIX_PATH="$QT_ROOT" \
    -DQt6_DIR="$Qt6_DIR"
if [ $? -ne 0 ]; then
    echo "CMake configuration failed!"
    exit 1
fi
echo ""

# Step 3: Build Qt application
echo "Step 3: Building Qt application..."
cmake --build .
if [ $? -ne 0 ]; then
    echo ""
    echo "=================================="
    echo "QT BUILD FAILED"
    echo "=================================="
    exit 1
fi
echo ""

echo "=================================="
echo "BUILD COMPLETE!"
echo "=================================="
echo ""
echo "Executable: $BUILD_DIR/myme-qt"
echo "Run with: $BUILD_DIR/myme-qt"
echo ""
echo "For a portable build with bundled Qt, use linuxdeploy (see CI workflow)."
echo ""
