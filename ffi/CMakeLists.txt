cmake_minimum_required(VERSION 3.28)
project(ffi)

include(FetchContent)

set(CMAKE_CXX_STANDARD 17)

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
include_directories(ffi PRIVATE "../target/include" "../target/cxxbridge/libfreenect2-rs/src" "../target/cxxbridge" "include")
