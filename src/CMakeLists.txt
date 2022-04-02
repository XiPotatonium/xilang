find_package(fmt CONFIG)
find_package(pegtl CONFIG)
find_package(CLI11 CONFIG)

set(LANG_SRC
  lang/parser.cpp
)

# Generic test that uses conan libs
add_executable(xilang lang.cpp ${LANG_SRC})
target_link_libraries(
  xilang
  PRIVATE project_options
          project_warnings
          fmt::fmt
          taocpp::pegtl
          CLI11::CLI11)

target_include_directories(xilang PRIVATE "${CMAKE_BINARY_DIR}/configured_files/include")

