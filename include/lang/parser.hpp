#ifndef XILANG_LANG_PARSER_HPP
#define XILANG_LANG_PARSER_HPP

#include <string_view>
#include <tao/pegtl/file_input.hpp>

#include "lang/ast.hpp"

namespace lang {

class FileParser {
public:
    explicit FileParser(const std::string_view file) : input_(file), tree_(nullptr) {}

    const ParseTreeNode *Parse();

private:
    tao::pegtl::file_input<> input_;
    std::unique_ptr<ParseTreeNode> tree_;
};

}// namespace lang

#endif