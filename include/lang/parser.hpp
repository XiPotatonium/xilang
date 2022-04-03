#ifndef XILANG_LANG_GRAMMAR_HPP
#define XILANG_LANG_GRAMMAR_HPP

#include <string_view>
#include <tao/pegtl/contrib/parse_tree.hpp>

namespace lang {

std::unique_ptr<tao::pegtl::parse_tree::node> parse(std::string_view file);

}

#endif