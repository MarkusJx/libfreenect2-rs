cmake_minimum_required(VERSION 3.22)
project(ffi)

include(FetchContent)

set(CMAKE_CXX_STANDARD 17)

option(CMake_RUN_CLANG_TIDY "Run clang-tidy with the compiler." OFF)
if (CMake_RUN_CLANG_TIDY)
    if (CMake_SOURCE_DIR STREQUAL CMake_BINARY_DIR)
        message(FATAL_ERROR "CMake_RUN_CLANG_TIDY requires an out-of-source build!")
    endif ()
    find_program(CLANG_TIDY_COMMAND NAMES clang-tidy)
    if (NOT CLANG_TIDY_COMMAND)
        message(WARNING "CMake_RUN_CLANG_TIDY is ON but clang-tidy is not found!")
        set(CMAKE_CXX_CLANG_TIDY "" CACHE STRING "" FORCE)
    else ()
        set(CMAKE_CXX_CLANG_TIDY "${CLANG_TIDY_COMMAND};-header-filter='${CMAKE_SOURCE_DIR}/include/*'")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Werror")
    endif ()

    # Create a preprocessor definition that depends on .clang-tidy content so
    # the compile command will change when .clang-tidy changes.  This ensures
    # that a subsequent build re-runs clang-tidy on all sources even if they
    # do not otherwise need to be recompiled.  Nothing actually uses this
    # definition.  We add it to targets on which we run clang-tidy just to
    # get the build dependency on the .clang-tidy file.
    file(SHA1 ${CMAKE_CURRENT_SOURCE_DIR}/.clang-tidy clang_tidy_sha1)
    set(CLANG_TIDY_DEFINITIONS "CLANG_TIDY_SHA1=${clang_tidy_sha1}")
    unset(clang_tidy_sha1)
endif ()
configure_file(.clang-tidy .clang-tidy COPYONLY)

add_compile_definitions(LIBFREENECT2_RS_WITH_OPENCL)
add_compile_definitions(LIBFREENECT2_RS_WITH_OPENGL)

add_library(ffi STATIC
        src/frame.cpp
        include/frame.hpp
        src/libfreenect2.cpp
        include/libfreenect2.hpp
        src/config.cpp
        include/config.hpp
        src/freenect2_device.cpp
        include/freenect2_device.hpp
        src/registration.cpp
        include/registration.hpp
        src/logger.cpp
        include/logger.hpp)
include_directories(ffi PRIVATE "../target/include" "../target/cxxbridge/libfreenect2-rs/src" "../target/cxxbridge" "include")
