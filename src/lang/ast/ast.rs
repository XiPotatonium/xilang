use crate::ir::flag::*;

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
    New,
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
    Func(String, Box<AST>, Vec<Box<AST>>, Box<AST>),

    Field(String, Box<AST>, Flag),
    Param(String, Box<AST>, Flag),
    // pattern, ty, flag, init: Box<AST>
    Let(Box<AST>, Box<AST>, Flag, Box<AST>),

    // children: Vec<Stmt>
    Block(Vec<Box<AST>>),
    // cond: Box<Expr>, then: Box<Block>, els: Box<Stmt>
    If(Box<AST>, Box<AST>, Box<AST>),
    Loop(Box<AST>),

    // ret_val: Box<Expr>
    Return(Box<AST>),
    Continue,
    // break_val: Box<Expr>
    Break(Box<AST>),

    // op, op1: Box<Expr>
    Unary(Op, Box<AST>),
    Binary(Op, Box<AST>, Box<AST>),
    Cast(Box<AST>, Box<AST>),
    // f: Box<Expr>, ps: Vec<Expr>
    Call(Box<AST>, Vec<Box<AST>>),

    Id(String),
    TuplePattern(Vec<Box<AST>>),

    // Type
    BoolType,
    I32Type,
    F64Type,
    TupleType(Vec<Box<AST>>),
    // type, dim
    ArrType(Box<AST>, Box<AST>),
    // class names
    ClassType(Vec<String>),

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
