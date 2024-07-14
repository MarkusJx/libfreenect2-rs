cmake_minimum_required(VERSION 3.28)
project(ffi)

set(CMAKE_CXX_STANDARD 17)

if (NOT EXISTS "${CMAKE_SOURCE_DIR}/include/cxx.h")
    message(STATUS "Downloading cxx.h")
    file(DOWNLOAD
            "https://raw.githubusercontent.com/dtolnay/cxx/master/include/cxx.h"
            "${CMAKE_SOURCE_DIR}/include/cxx.h")
    message(STATUS "Downloading cxx.h - done")
else ()
    message(STATUS "cxx.h already exists, skipping download")
endif ()

add_library(ffi STATIC
        src/frame.cpp
        include/frame.hpp
        include/libfreenect2.hpp
        src/libfreenect2.cpp
        include/macros.hpp
        src/config.cpp
        include/config.hpp
        src/freenect2_device.cpp
        include/freenect2_device.hpp)
include_directories(ffi PUBLIC "C:\\Users\\marku\\Desktop\\libfreenect2\\build\\install\\include" "../target/cxxbridge/libfreenect2-rs/src" "include")
