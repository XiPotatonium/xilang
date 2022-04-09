#include <catch2/catch.hpp>
#include <cstdint>
#include <iostream>
#include <string_view>
#include <tao/pegtl/contrib/analyze.hpp>
#include <tao/pegtl/contrib/parse_tree.hpp>
#include <tao/pegtl/parse_error.hpp>

#include "lang/grammar.hpp"
#include "lang/parser.hpp"


bool test_parse(const std::string_view file)
{
    lang::FileParser parser {file};
    return parser.Parse() != nullptr;
}

TEST_CASE("Factorials are computed", "[factorial]") {
    REQUIRE(tao::pegtl::analyze<lang::grammar::Grammar>() == 0);
    REQUIRE(test_parse("../examples/arr.xi"));
    REQUIRE(test_parse("../examples/global.xi"));
    REQUIRE(test_parse("../examples/hello.xi"));
    REQUIRE(test_parse("../examples/math.xi"));
    REQUIRE(test_parse("../examples/template.xi"));

    REQUIRE(test_parse("../sys/mod.xi"));
    REQUIRE(test_parse("../sys/collections.xi"));
}
