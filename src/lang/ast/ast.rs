use super::expr::Op;
use crate::ir::flag::*;

#[derive(Debug)]
pub enum Type {
    Bool,
    Int,
    Double,
    Tuple(Vec<Type>),
    Arr(Box<Type>, usize),
    // class names
    Class(Vec<String>),
    Unk, // Type determined at compile time
}

impl Type {
    pub fn empty() -> Type {
        Type::Tuple(vec![])
    }
}

impl Clone for Type {
    fn clone(&self) -> Type {
        match self {
            Self::Bool => Self::Bool,
            Self::Int => Self::Int,
            Self::Double => Self::Double,
            Self::Tuple(v) => Self::Tuple(v.to_vec()),
            Self::Arr(dtype, dim) => Self::Arr(Box::new(dtype.as_ref().clone()), *dim),
            Self::Class(names) => Self::Class({
                let mut copy_names: Vec<String> = Vec::new();
                for name in names.iter() {
                    copy_names.push(name.clone());
                }
                copy_names
            }),
            Self::Unk => Self::Unk,
        }
    }
}

#[derive(Debug)]
pub enum AST {
    // classes: Vec<AST>
    File(Vec<AST>),

    // id, funcs: Vec<Func>, fields: Vec<Var>
    Class(String, Vec<AST>, Vec<AST>),
    // id, ty, ps: Vec<Var>, body: Box<Block>
    Func(String, Box<AST>, Vec<AST>, Box<AST>),
    // id, ty, flag, init: Box<AST>
    Var(String, Box<AST>, Flag, Box<AST>),
    // children: Vec<Stmt>
    Block(Vec<AST>),
    // cond: Box<Expr>, then: Box<Block>, els: Box<Stmt>
    If(Box<AST>, Box<AST>, Box<AST>),
    // ret_val: Box<Expr>
    Return(Box<AST>),
    // pattern: Box<AST>, iter: Box<Expr>, body: Box<Block>
    For(Box<AST>, Box<AST>, Box<AST>),
    // cond: Box<Expr>, body: Box<Block>
    While(Box<AST>, Box<AST>),
    Loop(Box<AST>),
    Continue,
    Break,

    // expr: Box<Expr>
    ExprStmt(Box<AST>),

    // op, op1: Box<Expr>
    Unary(Op, Box<AST>),
    Binary(Op, Box<AST>, Box<AST>),
    // f: Box<Expr>, ps: Vec<Expr>
    Call(Box<AST>, Vec<AST>),
    // type: Box<Type>
    New(Box<AST>),
    // type: Box<Expr>, val: Box<AST>
    Cast(Box<AST>, Box<AST>),

    // type: Type
    Type(Type),
    Id(String),

    // Literal
    Null,
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
    Char(u32),

    // Option<AST>::None
    None,
}
