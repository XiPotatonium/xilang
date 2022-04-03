#include <tao/pegtl/contrib/parse_tree.hpp>

#include "lang/cfg.hpp"
#include "lang/grammar.hpp"
#include "lang/parser.hpp"

using namespace tao::pegtl;

namespace lang {

std::unique_ptr<parse_tree::node> parse(const std::string_view file) {
    file_input input(file);
    auto root = parse_tree::parse<grammar::Grammar>(input);
    return root;
}

}// namespace lang