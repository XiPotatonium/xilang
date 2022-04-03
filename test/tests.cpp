#include <catch2/catch.hpp>
#include <cstdint>
#include <fmt/core.h>
#include <fmt/ostream.h>
#include <iostream>
#include <string_view>
#include <tao/pegtl/contrib/analyze.hpp>
#include <tao/pegtl/contrib/parse_tree.hpp>
#include <tao/pegtl/parse_error.hpp>

#include "lang/grammar.hpp"
#include "lang/parser.hpp"

bool test_parse(const std::string_view file)// NOLINT(misc-no-recursion)
{
    tao::pegtl::file_input input(file);
    const auto root = tao::pegtl::parse_tree::parse<lang::grammar::Grammar>(input);
    return root != nullptr;
}

TEST_CASE("Factorials are computed", "[factorial]") {
    REQUIRE(tao::pegtl::analyze<lang::grammar::Grammar>() == 0);
    REQUIRE(test_parse("../examples/main.xi"));
    REQUIRE(test_parse("../sys/mod.xi"));
}
