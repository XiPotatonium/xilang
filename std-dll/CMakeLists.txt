# CMakeList.txt : CMake project for std-dll, include source and define
# project specific logic here.
#
cmake_minimum_required (VERSION 3.8)

project ("std-dll")

# Add source to this project's executable.
add_library (std-dll SHARED "lib.c")

add_custom_command(
       TARGET std-dll 
       POST_BUILD
       COMMAND ${CMAKE_COMMAND} -E copy
           $<TARGET_FILE:std-dll>
           ${CMAKE_SOURCE_DIR}/../target/std/std.dll
)

# TODO: Add tests and install targets if needed.
