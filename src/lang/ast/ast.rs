use crate::ir::flag::*;

#[derive(Debug)]
pub enum Type {
    Bool,
    I32,
    F64,
    Tuple(Vec<Box<Type>>),
    Arr(Box<Type>, Box<AST>),
    // class names
    Class(Vec<String>),
    Unk, // Type determined at compile time
}

impl Type {
    pub fn empty() -> Type {
        Type::Tuple(vec![])
    }
}

/*
impl Clone for Type {
    fn clone(&self) -> Type {
        match self {
            Self::Bool => Self::Bool,
            Self::I32 => Self::I32,
            Self::F64 => Self::F64,
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
*/

#[derive(Debug)]
pub enum Op {
    Neg,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LogNot,
    LogAnd,
    LogOr,
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
    Assign,
    StaticAccess,
    ObjAccess,
    ArrayAccess,
}

#[derive(Debug)]
pub enum AST {
    // classes: Vec<AST>
    File(Vec<Box<AST>>),

    // id, methods: Vec<Func>, fields: Vec<Var>
    Class(String, Vec<Box<AST>>, Vec<Box<AST>>),
    // id, ty, ps: Vec<Var>, body: Box<Block>
    Func(String, Box<Type>, Vec<Box<AST>>, Box<AST>),

    Field(String, Box<Type>, Flag),
    Param(String, Box<Type>, Flag),
    // pattern, ty, flag, init: Box<AST>
    Var(Box<AST>, Box<Type>, Flag, Box<AST>),

    // children: Vec<Stmt>
    Block(bool, Vec<Box<AST>>),
    // cond: Box<Expr>, then: Box<Block>, els: Box<Stmt>
    If(bool, Box<AST>, Box<AST>, Box<AST>),
    Loop(bool, Box<AST>),
    // ret_val: Box<Expr>
    Return(Box<AST>),
    Continue,
    // break_val: Box<Expr>
    Break(Box<AST>),

    // expr: Box<Expr>
    ExprStmt(Box<AST>),

    // op, op1: Box<Expr>
    Unary(Op, Box<AST>),
    Binary(Op, Box<AST>, Box<AST>),
    Cast(Box<Type>, Box<AST>),
    // f: Box<Expr>, ps: Vec<Expr>
    Call(Box<AST>, Vec<Box<AST>>),
    // type: Box<Type>
    New(Box<Type>),

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

impl AST {
    pub fn is_expr(&self) -> bool {
        match self {
            Self::File(_) => false,
            Self::Class(_, _, _) => false,
            Self::Func(_, _, _, _) => false,
            Self::Field(_, _, _) => false,
            Self::Param(_, _, _) => false,
            Self::Var(_, _, _, _) => false,

            Self::Block(is_expr, _) => *is_expr,
            Self::If(is_expr, _, _, _) => *is_expr,
            Self::Loop(is_expr, _) => *is_expr,

            Self::Continue => false,
            Self::Return(_) => false,
            Self::Break(_) => false,

            Self::ExprStmt(_) => false,

            Self::Unary(_, _) => true,
            Self::Binary(_, _, _) => true,
            Self::Cast(_, _) => true,
            Self::Call(_, _) => true,
            Self::New(_) => true,

            Self::Id(_) => true,
            Self::Null => true,
            Self::Bool(_) => true,
            Self::Int(_) => true,
            Self::Float(_) => true,
            Self::String(_) => true,
            Self::Char(_) => true,

            // 注意None也是表达式，代表()
            Self::None => true,
        }
    }
}
