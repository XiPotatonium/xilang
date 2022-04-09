#include "lang/ast.hpp"
#include "core/common.hpp"
#include "core/path.hpp"
#include "lang/grammar.hpp"
#include <cassert>
#include <exception>
#include <fmt/core.h>
#include <memory>
#include <range/v3/view/take.hpp>

namespace lang {

/**
 * @brief Parsing grammar guarantees that the input path is in correct form
 *
 * @param node
 * @return core::PathBuf
 */
core::PathBuf BuildPath(const ParseTreeNode &node) {
    core::PathBuf path;
    for (const auto &seg : node.children) {
        if (seg->is_type<grammar::Dot>()) {
            path.push(core::PathSeg {"."});
        } else if (seg->is_type<grammar::IdG>()) {
            const auto &id = seg->children[0]->string_view();
            if (seg->children.size() == 1) {
                path.push(core::PathSeg {id});
            } else {
                std::vector<core::PathBuf> generics;
                //cppcheck-suppress useStlAlgorithm
                for (const auto &ty : seg->children | ranges::views::take(1)) { generics.push_back(BuildPath(*ty)); }
                path.push(core::PathSeg {id, std::move(generics)});
            }
        } else {
            std::unexpected();
        }
    }
    return path;
}

std::unique_ptr<ASTType> InitType(const ParseTreeNode &node) {
    const auto &non_arr_ty = node.children[0];
    bool is_arr = node.children.size() != 1;
    if (non_arr_ty->is_type<grammar::KwF32>()) {
        return std::make_unique<ASTType>(node, ASTType::Type::kF32, is_arr);
    } else if (non_arr_ty->is_type<grammar::KwUSelf>()) {
        return std::make_unique<ASTType>(node, ASTType::Type::kSelf, is_arr);
    } else if (non_arr_ty->is_type<grammar::Path>()) {
        auto path = BuildPath(*non_arr_ty.get());
        return std::make_unique<ASTPathType>(node, std::move(path), is_arr);
    } else if (non_arr_ty->is_type<grammar::TupleType>()) {
        throw core::NotImplementedException();
    } else {
        std::unexpected();
    }
}

std::unique_ptr<ASTGlobal> InitGlobal(const ParseTreeNode &node) {
    assert(node.children[1]->is_type<grammar::Id>());
    assert(node.children[1]->is_type<grammar::Type>());
    return std::make_unique<ASTGlobal>(node, node.children[0]->string_view(), node.children[1]->data<ASTType>());
}

std::unique_ptr<ASTUseStmt> InitUseStmtAST(const ParseTreeNode &node) {
    auto data = std::make_unique<ASTUseStmt>(node);

    return data;
}

std::unique_ptr<ASTFile> InitFileAST(const ParseTreeNode &node) {
    auto data = std::make_unique<ASTFile>(node);

    for (const auto &child : node.children) {
        if (child->is_type<grammar::UseStmt>()) {
            data->uses.push_back(child->data<ASTUseStmt>());
        } else if (child->is_type<grammar::Fn<>>()) {
            data->fns.push_back(child->data<ASTFn>());
        } else if (child->is_type<grammar::Struct>()) {
            data->structs.push_back(child->data<ASTStruct>());
        } else if (child->is_type<grammar::Interface>()) {
            data->interfaces.push_back(child->data<ASTInterface>());
        } else if (child->is_type<grammar::Global>()) {
            data->globals.push_back(child->data<ASTGlobal>());
        } else if (child->is_type<grammar::Enum>()) {
            data->enums.push_back(child->data<ASTEnum>());
        } else {
            std::unexpected();
        }
    }

    return data;
}

void ParseTreeNode::InitData() {
    if (this->is_type<grammar::Type>()) {
        this->data_ = InitType(*this);
    } else if (this->is_type<grammar::Global>()) {
        this->data_ = InitGlobal(*this);
    } else if (this->is_type<grammar::UseStmt>()) {
        this->data_ = InitUseStmtAST(*this);
    } else if (this->is_type<grammar::Grammar>()) {
        this->data_ = InitFileAST(*this);
    } else {
        std::unexpected();
    }
}

}// namespace lang