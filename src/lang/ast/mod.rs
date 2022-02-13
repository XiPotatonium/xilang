mod disp;
mod member;
mod strukt;
mod ty;

use super::util::ItemPathBuf;
pub use core::flags::{ClassFlag, ClassFlags, FieldFlag, FieldFlags, MethodFlag, MethodFlags};
pub use member::{ASTCtor, ASTField, ASTMethod};
pub use strukt::{ASTStruct, ASTStructFieldInit};
pub use ty::ASTType;

pub enum AST {
    /// mods, classes/interfaces
    File(Vec<String>, Vec<Box<AST>>),

    /// path, as
    Use(ItemPathBuf, Option<String>),

    /// attrib name, args
    CustomAttrib(String, Vec<Box<AST>>),

    Struct(ASTStruct),

    Method(ASTMethod),
    Field(ASTField),

    Param(String, Box<ASTType>),
    /// pattern, attrib, ty, init: Box<AST>
    Let(Box<AST>, Box<ASTType>, Box<AST>),

    ExprStmt(Box<AST>),

    /// children: Vec<Stmt>
    Block(Vec<Box<AST>>),
    /// cond: Box<Expr>, then: Box<Block>, els: Box<Stmt>
    If(Box<AST>, Box<AST>, Box<AST>),
    Loop(Box<AST>),

    /// ret_val: Box<Expr>
    Return(Box<AST>),
    Continue,
    /// break_val: Box<Expr>
    Break(Box<AST>),

    OpPos(Box<AST>),
    OpNeg(Box<AST>),
    OpAdd(Box<AST>, Box<AST>),
    OpSub(Box<AST>, Box<AST>),
    OpMul(Box<AST>, Box<AST>),
    OpDiv(Box<AST>, Box<AST>),
    OpMod(Box<AST>, Box<AST>),
    OpLogNot(Box<AST>),
    OpLogAnd(Box<AST>, Box<AST>),
    OpLogOr(Box<AST>, Box<AST>),
    OpEq(Box<AST>, Box<AST>),
    OpNe(Box<AST>, Box<AST>),
    OpGe(Box<AST>, Box<AST>),
    OpGt(Box<AST>, Box<AST>),
    OpLe(Box<AST>, Box<AST>),
    OpLt(Box<AST>, Box<AST>),
    OpAssign(Box<AST>, Box<AST>),
    OpStaticAccess(Box<AST>, String),
    OpObjAccess(Box<AST>, String),
    OpArrayAccess(Box<AST>, Box<AST>),
    /// ty, val
    OpCast(Box<ASTType>, Box<AST>),
    /// f: Box<Expr>, ps: Vec<Expr>
    OpCall(Box<AST>, Vec<Box<AST>>),
    /// ty, ps: Vec<Expr>
    OpNewStruct(Box<ASTType>, Vec<Box<ASTStructFieldInit>>),
    /// elem_ty, dim: Expr
    OpNewArr(Box<ASTType>, Box<AST>),

    Id(String),
    TuplePattern(Vec<Box<AST>>),

    Type(Box<ASTType>),

    /// Literal
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
    Char(u32),

    /// Option<AST>::None
    None,
}
