#ifndef XILANG_LANG_AST_HPP
#define XILANG_LANG_AST_HPP

#include <cstddef>
#include <cstdint>
#include <fmt/core.h>
#include <fmt/ostream.h>
#include <functional>
#include <memory>
#include <optional>
#include <ostream>
// #include <range/v3/range.hpp>
// #include <range/v3/range_fwd.hpp>
// #include <range/v3/view.hpp>
#include <range/v3/view/enumerate.hpp>
#include <string_view>
#include <tao/pegtl.hpp>
#include <tao/pegtl/contrib/parse_tree.hpp>
#include <type_traits>
#include <vector>

#include "core/path.hpp"

namespace lang {

template<typename T>
using ASTChildrenLst = std::vector<std::reference_wrapper<const T>>;

struct AbstractAST;

struct ParseTreeNode : tao::pegtl::parse_tree::basic_node<ParseTreeNode> {

    std::size_t line() noexcept { return this->begin().line; }

    std::size_t column() noexcept { return this->begin().column; }

    template<typename T>
    requires std::is_base_of_v<AbstractAST, T>
    const T &data() {
        if (this->data_ == nullptr) { this->InitData(); }
        return dynamic_cast<T &>(*this->data_);
    }

private:
    void InitData();

    std::unique_ptr<AbstractAST> data_;
};

struct AbstractAST {
    const ParseTreeNode &node;

    explicit AbstractAST(const ParseTreeNode &node) : node(node) {}
    virtual ~AbstractAST() = default;

    virtual void Display(std::ostream &os) const = 0;

    // template<typename T>
    // static void Iter(std::vector<const ASTNode *> &nodes) {
    //     // NOTE: dynamic_cast misuse?
    //     auto rng = nodes | ranges::views::transform([](const ASTNode *node) { return dynamic_cast<T &>(*node->data);
    //     }); return rng;
    // }
};

struct ASTType : AbstractAST {
    enum class Type {
        kBool,
        kChar,
        kF32,
        kF64,
        kISize,
        kI32,
        kI64,
        kUSize,
        kStr,
        kU8,
        kU32,
        kU64,
        kSelf,
        kPath,
        kTuple,
    };
    Type ty;
    bool is_arr;

    ASTType(const ParseTreeNode &node, Type ty, bool is_arr) : AbstractAST(node), ty(ty), is_arr(is_arr) {}
    void Display(std::ostream &os) const override {
        os << "Type ";
        switch (ty) {
        case Type::kBool:
            os << "bool";
            break;
        case Type::kChar:
            os << "char";
            break;
        case Type::kF32:
            os << "f32";
            break;
        case Type::kF64:
            os << "f64";
            break;
        case Type::kISize:
            os << "isize";
            break;
        case Type::kI32:
            os << "i32";
            break;
        case Type::kI64:
            os << "i64";
            break;
        case Type::kUSize:
            os << "isize";
            break;
        case Type::kStr:
            os << "str";
            break;
        case Type::kU8:
            os << "u8";
            break;
        case Type::kU32:
            os << "u32";
            break;
        case Type::kU64:
            os << "u64";
            break;
        case Type::kSelf:
            os << "Self";
            break;
        case Type::kPath:
            os << "Path";
            break;
        case Type::kTuple:
            os << "Tuple";
            break;
        }
        if (is_arr) { os << "[]"; }
    }
};

struct ASTPathType : ASTType {
    core::PathBuf path;

    ASTPathType(const ParseTreeNode &node, core::PathBuf&& path, bool is_arr) : ASTType(node, Type::kPath, is_arr), path(path) {}

    void Display(std::ostream &os) const override {
        fmt::print(os, "Type {}", this->path.string());
        if (is_arr) { os << "[]"; }
    }
};

struct ASTTupleType : ASTType {
    ASTChildrenLst<ASTType> types;

    explicit ASTTupleType(const ParseTreeNode &node, ASTChildrenLst<ASTType>&& types, bool is_arr) : ASTType(node, Type::kTuple, is_arr), types(types) {}
    void Display(std::ostream &os) const override {
        os << "Type (";
        for (const auto &[i, v] : types | ranges::views::enumerate) {
            if (i != 0) { os << ", "; }
            v.get().Display(os);
        }
        os << ")";
        if (is_arr) { os << "[]"; }
    }
};

struct ASTAttrib : AbstractAST {
    std::string_view id;
    std::optional<std::vector<const AbstractAST *>> args;

    explicit ASTAttrib(const ParseTreeNode &node) : AbstractAST(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "Attrib {}", this->id); }
};

struct ASTFn : AbstractAST {
    ASTChildrenLst<ASTAttrib> attribs;
    std::string_view id;
    std::vector<std::string_view> generics;
    const ASTType *ret{};
    ASTChildrenLst<AbstractAST> ps;
    const AbstractAST *body{};

    explicit ASTFn(const ParseTreeNode &node) : AbstractAST(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "Fn {}", this->id); }
};

struct ASTMethod : ASTFn {
    explicit ASTMethod(const ParseTreeNode &node) : ASTFn(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "Method {}", this->id); }
};

struct ASTField : AbstractAST {
    std::string_view id;
    const ASTType &type;

    ASTField(const ParseTreeNode &node, const std::string_view &id, const ASTType &type)
        : AbstractAST(node), id(id), type(type) {}
    void Display(std::ostream &os) const override { fmt::print(os, "Field {}", this->id); }
};

struct ASTInterface : AbstractAST {
    ASTChildrenLst<ASTAttrib> attribs;
    ASTChildrenLst<ASTMethod> methods;

    std::string_view id;
    std::vector<std::string_view> generics;
    std::vector<core::PathBuf> impls;

    explicit ASTInterface(const ParseTreeNode &node) : AbstractAST(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "Interface {}", this->id); }
};

struct ASTGlobal : AbstractAST {
    std::string_view id;
    const ASTType &type;

    ASTGlobal(const ParseTreeNode &node, const std::string_view &id, const ASTType &type)
        : AbstractAST(node), id(id), type(type) {}
    void Display(std::ostream &os) const override { fmt::print(os, "Global {}", this->id); }
};

struct ASTStruct : AbstractAST {
    ASTChildrenLst<ASTAttrib> attribs;
    ASTChildrenLst<ASTField> fields;
    ASTChildrenLst<ASTFn> fns;
    ASTChildrenLst<ASTMethod> methods;

    std::string_view id;
    std::vector<std::string_view> generics;
    std::vector<core::PathBuf> impls;

    explicit ASTStruct(const ParseTreeNode &node) : AbstractAST(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "Struct {}", this->id); }
};

struct ASTEnumField : AbstractAST {
    std::string_view id;
    const ASTType *type {};

    explicit ASTEnumField(const ParseTreeNode &node) : AbstractAST(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "EnumField {}", this->id); }
};

struct ASTEnum : AbstractAST {
    ASTChildrenLst<ASTAttrib> attribs;
    ASTChildrenLst<ASTField> fields;
    ASTChildrenLst<ASTFn> fns;
    ASTChildrenLst<ASTMethod> methods;

    std::string_view id;
    std::vector<std::string_view> generics;

    explicit ASTEnum(const ParseTreeNode &node) : AbstractAST(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "Enum {}", this->id); }
};

struct ASTUseStmt : AbstractAST {
    core::PathBuf path;
    std::string_view id;

    explicit ASTUseStmt(const ParseTreeNode &node) : AbstractAST(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "UseStmt {}", this->id); }
};

struct ASTFile : AbstractAST {
    ASTChildrenLst<ASTUseStmt> uses;
    ASTChildrenLst<ASTInterface> interfaces;
    ASTChildrenLst<ASTStruct> structs;
    ASTChildrenLst<ASTEnum> enums;
    ASTChildrenLst<ASTFn> fns;
    ASTChildrenLst<ASTGlobal> globals;

    explicit ASTFile(const ParseTreeNode &node) : AbstractAST(node) {}
    void Display(std::ostream &os) const override { fmt::print(os, "File"); }
};

}// namespace lang

#endif
