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
    set(CARGO_BUILD_TYPE "")
else()
    set(RUST_LIB_DIR "${RUST_TARGET_DIR}/release")
    set(CARGO_BUILD_TYPE "--release")
endif()

# Set Qt path for cxx-qt build
set(QT_PATH "C:/Qt/6.10.1/msvc2022_64")
if(WIN32)
    set(QMAKE_PATH "${QT_PATH}/bin/qmake.exe")
else()
    set(QMAKE_PATH "${QT_PATH}/bin/qmake")
endif()

# Build Rust library with Qt environment
if(WIN32)
    set(RUST_LIB_OUTPUT "${RUST_LIB_DIR}/myme_ui.lib")
else()
    set(RUST_LIB_OUTPUT "${RUST_LIB_DIR}/libmyme_ui.a")
endif()

add_custom_command(
    OUTPUT ${RUST_LIB_OUTPUT}
    COMMAND ${CMAKE_COMMAND} -E env
        "QMAKE=${QMAKE_PATH}"
        "CMAKE_PREFIX_PATH=${QT_PATH}"
        "Qt6_DIR=${QT_PATH}/lib/cmake/Qt6"
        cargo build -p myme-ui ${CARGO_BUILD_TYPE}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    COMMENT "Building Rust myme-ui library with cxx-qt..."
    VERBATIM
)

add_custom_target(rust_myme_ui DEPENDS ${RUST_LIB_OUTPUT})

# Main executable
add_executable(myme-qt
    qt-main/main.cpp
)

# Make sure Rust library is built before linking
add_dependencies(myme-qt rust_myme_ui)

# Link Qt libraries
target_link_libraries(myme-qt PRIVATE
    Qt6::Core
    Qt6::Quick
    Qt6::QuickControls2
)

# Link Rust static library (after cargo build)
# On Windows, the extension is .lib not .a
if(WIN32)
    target_link_libraries(myme-qt PRIVATE
        "${RUST_LIB_DIR}/myme_ui.lib"
    )
else()
    target_link_libraries(myme-qt PRIVATE
        "${RUST_LIB_DIR}/libmyme_ui.a"
    )
endif()

# Find cxx-qt generated files by looking for the generated library
if(WIN32)
    file(GLOB_RECURSE CXX_QT_GEN_LIB "${RUST_LIB_DIR}/build/myme-ui-*/out/myme-ui-cxxqt-generated.lib")
else()
    file(GLOB_RECURSE CXX_QT_GEN_LIB "${RUST_LIB_DIR}/build/myme-ui-*/out/libmyme-ui-cxxqt-generated.a")
endif()

if(CXX_QT_GEN_LIB)
    list(GET CXX_QT_GEN_LIB 0 CXX_QT_GEN_LIB)
    get_filename_component(CXX_QT_OUT_DIR "${CXX_QT_GEN_LIB}" DIRECTORY)
    message(STATUS "Found cxx-qt generated files: ${CXX_QT_OUT_DIR}")

    # Include headers from the out directory
    target_include_directories(myme-qt PRIVATE "${CXX_QT_OUT_DIR}")

    # Find cxxqtgen include directory
    get_filename_component(CXX_QT_BUILD_DIR "${CXX_QT_OUT_DIR}" DIRECTORY)
    target_include_directories(myme-qt PRIVATE "${CXX_QT_BUILD_DIR}/out/cxxqtgen")

    # Link cxx-qt generated libraries
    if(WIN32)
        target_link_libraries(myme-qt PRIVATE
            "${CXX_QT_OUT_DIR}/myme-ui-cxxqt-generated.lib"
            "${CXX_QT_OUT_DIR}/cxx-qt-call-init-crate_myme_ui.lib"
        )
    else()
        target_link_libraries(myme-qt PRIVATE
            "${CXX_QT_OUT_DIR}/libmyme-ui-cxxqt-generated.a"
            "${CXX_QT_OUT_DIR}/libcxx-qt-call-init-crate_myme_ui.a"
        )
    endif()
else()
    message(FATAL_ERROR "Could not find cxx-qt generated libraries. Did cargo build succeed?")
endif()

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
