mod class;
mod disp;
mod generic;
mod method;
mod ty;

use xir::attrib::*;

use super::util::ItemPathBuf;
pub use class::ASTClass;
pub use generic::{ASTGenericParamDecl, ASTIdWithGenericParam};
pub use method::{ASTCtor, ASTMethod, ASTMethodAttrib, ASTMethodAttribFlag};
pub use ty::ASTType;

pub enum AST {
    /// mods, ext_mods, uses, classes: Vec<AST>
    File(Vec<String>, Vec<String>, Vec<Box<AST>>, Vec<Box<AST>>),

    /// path, as
    Use(ItemPathBuf, Option<String>),

    /// attrib name, args
    CustomAttrib(String, Vec<Box<AST>>),

    Class(ASTClass),
    Struct(ASTClass),
    Ctor(ASTCtor),
    Method(ASTMethod),

    /// id, attrib, custom-attrib, ty
    Field(String, FieldAttrib, Vec<Box<AST>>, Box<ASTType>),
    Param(String, ParamAttrib, Box<ASTType>),
    /// pattern, attrib, ty, init: Box<AST>
    Let(Box<AST>, LocalAttrib, Box<ASTType>, Box<AST>),

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
    OpStaticAccess(Box<AST>, ASTIdWithGenericParam),
    OpObjAccess(Box<AST>, ASTIdWithGenericParam),
    OpArrayAccess(Box<AST>, Box<AST>),
    /// ty, val
    OpCast(Box<ASTType>, Box<AST>),
    /// f: Box<Expr>, ps: Vec<Expr>
    OpCall(Box<AST>, Vec<Box<AST>>),
    /// ty, ps: Vec<Expr>
    OpNew(Box<ASTType>, Vec<Box<AST>>),
    /// ty, dim: Expr
    OpNewArr(Box<ASTType>, Box<AST>),

    Id(String),
    IdWithGenericParams(ASTIdWithGenericParam),
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
