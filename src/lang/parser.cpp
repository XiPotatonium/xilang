#include <tao/pegtl/contrib/parse_tree.hpp>

#include "lang/ast.hpp"
#include "lang/grammar.hpp"
#include "lang/parser.hpp"
#include "lang/selector.hpp"

using namespace tao::pegtl;

namespace lang {

const ParseTreeNode *FileParser::Parse() {
    if (this->tree_ == nullptr) {
        this->tree_ = parse_tree::parse<grammar::Grammar, ParseTreeNode, grammar::Selector>(this->input_);
    }
    return this->tree_.get();
}


}// namespace lang
