cmake_minimum_required(VERSION 3.16)

project(myme VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Enable Qt's automatic handling of resource files
set(CMAKE_AUTORCC ON)

# Find Qt
find_package(Qt6 REQUIRED COMPONENTS Core Quick QuickControls2)

# Resolve Qt path automatically from the found installation
get_target_property(QT_QMAKE_EXECUTABLE Qt6::qmake IMPORTED_LOCATION)
get_filename_component(QT_BIN_DIR "${QT_QMAKE_EXECUTABLE}" DIRECTORY)
get_filename_component(QT_PATH "${QT_BIN_DIR}" DIRECTORY)

message(STATUS "Using Qt from: ${QT_PATH}")
message(STATUS "qmake: ${QT_QMAKE_EXECUTABLE}")

# Rust target directory
set(RUST_TARGET_DIR "${CMAKE_SOURCE_DIR}/target")

# Handle multi-config generators (MSVC) vs single-config (Makefiles)
get_property(IS_MULTI_CONFIG GLOBAL PROPERTY GENERATOR_IS_MULTI_CONFIG)

if(IS_MULTI_CONFIG)
    # Multi-config: use generator expressions
    set(RUST_LIB_DIR_DEBUG "${RUST_TARGET_DIR}/debug")
    set(RUST_LIB_DIR_RELEASE "${RUST_TARGET_DIR}/release")
    set(RUST_LIB_DIR "$<IF:$<CONFIG:Debug>,${RUST_LIB_DIR_DEBUG},${RUST_LIB_DIR_RELEASE}>")
    set(CARGO_BUILD_TYPE "$<IF:$<CONFIG:Debug>,,--release>")
else()
    # Single-config: use CMAKE_BUILD_TYPE
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        set(RUST_LIB_DIR "${RUST_TARGET_DIR}/debug")
        set(CARGO_BUILD_TYPE "")
    else()
        set(RUST_LIB_DIR "${RUST_TARGET_DIR}/release")
        set(CARGO_BUILD_TYPE "--release")
    endif()
endif()

# Platform-specific library names
if(WIN32)
    set(RUST_LIB_NAME "myme_ui.lib")
    set(CXX_QT_GEN_PATTERN "myme-ui-cxxqt-generated.lib")
    set(CXX_QT_INIT_PATTERN "cxx-qt-call-init-crate_myme_ui.lib")
else()
    set(RUST_LIB_NAME "libmyme_ui.a")
    set(CXX_QT_GEN_PATTERN "libmyme-ui-cxxqt-generated.a")
    set(CXX_QT_INIT_PATTERN "libcxx-qt-call-init-crate_myme_ui.a")
endif()

# For multi-config, we need concrete paths for BYPRODUCTS
if(IS_MULTI_CONFIG)
    set(RUST_LIB_OUTPUT_DEBUG "${RUST_LIB_DIR_DEBUG}/${RUST_LIB_NAME}")
    set(RUST_LIB_OUTPUT_RELEASE "${RUST_LIB_DIR_RELEASE}/${RUST_LIB_NAME}")
    set(RUST_BYPRODUCTS ${RUST_LIB_OUTPUT_DEBUG} ${RUST_LIB_OUTPUT_RELEASE})
else()
    set(RUST_LIB_OUTPUT "${RUST_LIB_DIR}/${RUST_LIB_NAME}")
    set(RUST_BYPRODUCTS ${RUST_LIB_OUTPUT})
endif()

# Build Rust library with Qt environment
add_custom_target(rust_myme_ui ALL
    COMMAND ${CMAKE_COMMAND} -E env
        "QMAKE=${QT_QMAKE_EXECUTABLE}"
        "CMAKE_PREFIX_PATH=${QT_PATH}"
        "Qt6_DIR=${QT_PATH}/lib/cmake/Qt6"
        cargo build -p myme-ui ${CARGO_BUILD_TYPE}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    COMMENT "Building Rust myme-ui library with cxx-qt..."
    VERBATIM
    BYPRODUCTS ${RUST_BYPRODUCTS}
)

# Main executable with QML resources
set(APP_SOURCES
    qt-main/main.cpp
    qml.qrc
)

# Add Windows application icon
if(WIN32)
    list(APPEND APP_SOURCES assets/app.rc)
endif()

add_executable(myme-qt ${APP_SOURCES})

# Make sure Rust library is built before linking
add_dependencies(myme-qt rust_myme_ui)

# Link Qt libraries
target_link_libraries(myme-qt PRIVATE
    Qt6::Core
    Qt6::Quick
    Qt6::QuickControls2
)

# Determine which Rust lib dir to search based on generator type
if(IS_MULTI_CONFIG)
    # For multi-config, search release first (more common for actual builds)
    set(RUST_SEARCH_DIR "${RUST_LIB_DIR_RELEASE}")
else()
    set(RUST_SEARCH_DIR "${RUST_LIB_DIR}")
endif()

# Find cxx-qt generated files
file(GLOB_RECURSE CXX_QT_GEN_LIB "${RUST_SEARCH_DIR}/build/myme-ui-*/out/${CXX_QT_GEN_PATTERN}")

if(CXX_QT_GEN_LIB)
    list(GET CXX_QT_GEN_LIB 0 CXX_QT_GEN_LIB)
    get_filename_component(CXX_QT_OUT_DIR "${CXX_QT_GEN_LIB}" DIRECTORY)
    message(STATUS "Found cxx-qt generated files: ${CXX_QT_OUT_DIR}")

    # Include headers from the out directory
    target_include_directories(myme-qt PRIVATE "${CXX_QT_OUT_DIR}")

    # Find cxxqtgen include directory
    get_filename_component(CXX_QT_BUILD_DIR "${CXX_QT_OUT_DIR}" DIRECTORY)
    target_include_directories(myme-qt PRIVATE "${CXX_QT_BUILD_DIR}/out/cxxqtgen")

    # Link Rust static library and cxx-qt generated libraries
    if(WIN32)
        target_link_libraries(myme-qt PRIVATE
            "${RUST_SEARCH_DIR}/${RUST_LIB_NAME}"
            "${CXX_QT_OUT_DIR}/${CXX_QT_GEN_PATTERN}"
            "${CXX_QT_OUT_DIR}/${CXX_QT_INIT_PATTERN}"
            # Windows system libraries required by Rust dependencies
            ntdll
            ws2_32
            userenv
            advapi32
            bcrypt
            crypt32
            secur32
            Shlwapi
        )
    else()
        target_link_libraries(myme-qt PRIVATE
            "${RUST_SEARCH_DIR}/${RUST_LIB_NAME}"
            "${CXX_QT_OUT_DIR}/${CXX_QT_GEN_PATTERN}"
            "${CXX_QT_OUT_DIR}/${CXX_QT_INIT_PATTERN}"
        )
    endif()
else()
    message(FATAL_ERROR "Could not find cxx-qt generated libraries. Run 'cargo build -p myme-ui --release' first.")
endif()

# Set QML import paths
set(QML_IMPORT_PATH "${CMAKE_SOURCE_DIR}/crates/myme-ui/qml" CACHE STRING "" FORCE)
set(QML2_IMPORT_PATH "${CMAKE_SOURCE_DIR}/crates/myme-ui/qml" CACHE STRING "" FORCE)

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
