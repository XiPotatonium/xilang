find_package(fmt CONFIG)
find_package(pegtl CONFIG)
find_package(CLI11 CONFIG)
find_package(range-v3 CONFIG)

set(LANG_SRCS
  lang/parser.cpp
  lang/ast.cpp
)

set(CORE_SRCS
  core/path.cpp
)


# Generic test that uses conan libs
add_executable(xilang lang.cpp ${LANG_SRCS} ${CORE_SRCS})
target_link_libraries(
  xilang
  PRIVATE project_options
          project_warnings
          fmt::fmt
          taocpp::pegtl
          CLI11::CLI11
          range-v3::range-v3)

target_include_directories(xilang PRIVATE "${CMAKE_BINARY_DIR}/configured_files/include")
