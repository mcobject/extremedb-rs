# This is a helper CMake project to build the library independently from
# the Rust code.

cmake_minimum_required(VERSION 3.14)

project(cppwrapper)

set(MCO_ROOT "" CACHE PATH "eXtremeDB root directory")

add_library(cppwrapper STATIC rssql.cpp)

target_include_directories(cppwrapper PRIVATE ${MCO_ROOT}/include)
target_compile_options(cppwrapper PRIVATE -Wall -pedantic -Wextra)
