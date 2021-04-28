# CMakeList.txt : CMake project for std-dll, include source and define
# project specific logic here.
#
cmake_minimum_required (VERSION 3.8)

project ("xtd-native")

# Add source to this project's executable.
add_library (xtd SHARED "lib.c")

add_custom_command(
       TARGET xtd 
       POST_BUILD
       COMMAND ${CMAKE_COMMAND} -E copy
           $<TARGET_FILE:xtd>
           ${CMAKE_SOURCE_DIR}/../target/std/
)

# TODO: Add tests and install targets if needed.
