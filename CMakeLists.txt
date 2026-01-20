cmake_minimum_required(VERSION 3.16)

project(myme VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find Qt
find_package(Qt6 REQUIRED COMPONENTS Core Quick QuickControls2)

# Add Rust library
# The cxx-qt build will generate the necessary bridge code
set(RUST_TARGET_DIR "${CMAKE_SOURCE_DIR}/target")

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_LIB_DIR "${RUST_TARGET_DIR}/debug")
else()
    set(RUST_LIB_DIR "${RUST_TARGET_DIR}/release")
endif()

# Main executable
add_executable(myme-qt
    qt-main/main.cpp
)

# Link Qt libraries
target_link_libraries(myme-qt PRIVATE
    Qt6::Core
    Qt6::Quick
    Qt6::QuickControls2
)

# Link Rust static library (after cargo build)
target_link_libraries(myme-qt PRIVATE
    "${RUST_LIB_DIR}/libmyme_ui.a"
)

# Include cxx-qt generated headers
target_include_directories(myme-qt PRIVATE
    "${RUST_TARGET_DIR}/cxxqt/myme-ui"
)

# Set QML import path
set(QML_IMPORT_PATH "${CMAKE_SOURCE_DIR}/crates/myme-ui/qml" CACHE STRING "" FORCE)

# Qt properties
set_target_properties(myme-qt PROPERTIES
    WIN32_EXECUTABLE TRUE
    MACOSX_BUNDLE TRUE
)

# Install
install(TARGETS myme-qt
    BUNDLE DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)
