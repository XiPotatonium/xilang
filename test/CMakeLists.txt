find_package(Catch2 REQUIRED)
find_package(pegtl CONFIG)
find_package(fmt CONFIG)
find_package(range-v3 CONFIG)

include(CTest)
include(Catch)

add_library(catch_main OBJECT catch_main.cpp)
target_link_libraries(catch_main PUBLIC Catch2::Catch2)
target_link_libraries(catch_main PRIVATE project_options)

# Provide a simple smoke test to make sure that the CLI works and can display a --help message
add_test(NAME cli.has_help COMMAND xilang --help)

# Provide a test to verify that the version being reported from the application
# matches the version given to CMake. This will be important once you package
# your program. Real world shows that this is the kind of simple mistake that is easy
# to make, but also easy to test for.
add_test(NAME cli.version_matches COMMAND xilang --version)
set_tests_properties(cli.version_matches PROPERTIES PASS_REGULAR_EXPRESSION "${PROJECT_VERSION}")


add_executable(tests tests.cpp)
target_link_libraries(tests PRIVATE project_warnings project_options catch_main)

# automatically discover tests that are defined in catch based test files you can modify the unittests. Set TEST_PREFIX
# to whatever you want, or use different for different binaries
catch_discover_tests(
  tests
  TEST_PREFIX
  "unittests."
  REPORTER
  xml
  OUTPUT_DIR
  .
  OUTPUT_PREFIX
  "unittests."
  OUTPUT_SUFFIX
  .xml)


set (GRAMMAR_TEST_SRC
  ../src/lang/parser.cpp
  ../src/lang/ast.cpp
  ../src/core/path.cpp
)

# Add a file containing a set of constexpr tests
add_executable(grammar_tests grammar_tests.cpp ${GRAMMAR_TEST_SRC})
target_link_libraries(grammar_tests PRIVATE project_options project_warnings catch_main taocpp::pegtl fmt::fmt range-v3::range-v3)

catch_discover_tests(
  grammar_tests
  TEST_PREFIX
  "grammar."
  REPORTER
  xml
  OUTPUT_DIR
  .
  OUTPUT_PREFIX
  "grammar."
  OUTPUT_SUFFIX
  .xml)

# Disable the constexpr portion of the test, and build again this allows us to have an executable that we can debug when
# things go wrong with the constexpr testing
add_executable(relaxed_grammar_tests grammar_tests.cpp ${GRAMMAR_TEST_SRC})
target_link_libraries(relaxed_grammar_tests PRIVATE project_options project_warnings catch_main taocpp::pegtl fmt::fmt range-v3::range-v3)
target_compile_definitions(relaxed_grammar_tests PRIVATE -DCATCH_CONFIG_RUNTIME_STATIC_REQUIRE)

catch_discover_tests(
  relaxed_grammar_tests
  TEST_PREFIX
  "relaxed_grammar."
  REPORTER
  xml
  OUTPUT_DIR
  .
  OUTPUT_PREFIX
  "relaxed_grammar."
  OUTPUT_SUFFIX
  .xml)
