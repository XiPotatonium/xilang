#ifndef XILANG_LANG_SELECTOR_HPP
#define XILANG_LANG_SELECTOR_HPP

#include "ast.hpp"
#include "grammar.hpp"
#include <exception>
#include <tao/pegtl.hpp>
#include <tao/pegtl/contrib/parse_tree.hpp>
#include <type_traits>


namespace lang::grammar {


template<typename Rule>
using Selector = tao::pegtl::parse_tree::selector<Rule,
    tao::pegtl::parse_tree::store_content::
        on<Fn<>, Method, Field, Global, Impls, Struct, Interface, EnumField, Enum, UseStmt, Grammar>,
    tao::pegtl::parse_tree::fold_one::on<BasicType, NonArrType, ExprWOBlock, ExprWBlock, Expr>>;
/*
template<>
struct Selector<Attrib> : std::true_type {
    static void transform(std::unique_ptr<tao::pegtl::parse_tree::node> &n) {}
};
template<>
struct Selector<AttribLst> : std::true_type {
    static void transform(std::unique_ptr<tao::pegtl::parse_tree::node> &n) {}
};

template<>
struct Selector<Fn<>> : std::true_type {
    static void transform(std::unique_ptr<tao::pegtl::parse_tree::node> &n) {}
};
template<>
struct Selector<Method> : std::true_type {
    static void transform(std::unique_ptr<tao::pegtl::parse_tree::node> &n) {}
};

template<>
struct Selector<Field> : std::true_type {
    static void transform(std::unique_ptr<tao::pegtl::parse_tree::node> &n) {}
};


template<>
struct Selector<Global> : std::true_type {};
template<>
struct Selector<Impls> : std::true_type {};

template<>
struct Selector<StructOrInterface> : std::true_type {
    static void transform(std::unique_ptr<tao::pegtl::parse_tree::node> &n) {}
};

template<>
struct Selector<UseStmt> : std::true_type {
    static void transform(std::unique_ptr<tao::pegtl::parse_tree::node> &n) {}
};

template<>
struct Selector<Grammar> : std::true_type {
    static void transform(std::unique_ptr<tao::pegtl::parse_tree::node> &n) {}
};
*/

}// namespace lang::grammar

#endif
