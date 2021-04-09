mod disp;

use xir::attrib::*;
use xir::util::path::ModPath;

pub enum AST {
    /// mods, ext_mods, uses, classes: Vec<AST>
    File(Vec<String>, Vec<String>, Vec<Box<AST>>, Vec<Box<AST>>),

    /// path, as
    Use(ModPath, Option<String>),

    CustomAttr(String, Vec<Box<AST>>),

    /// id, attrib, custom-attrib, methods: Vec<Func>, fields: Vec<Var>, static-init
    Class(
        String,
        TypeAttrib,
        Vec<Box<AST>>,
        Vec<Box<AST>>,
        Vec<Box<AST>>,
        Box<AST>,
    ),
    /// id, attrib, custom-attrib, ty, ps: Vec<Var>, body: Box<Block>
    Method(
        String,
        MethodAttrib,
        Vec<Box<AST>>,
        Box<AST>,
        Vec<Box<AST>>,
        Box<AST>,
    ),

    /// id, attrib, custom-attrib, ty
    Field(String, FieldAttrib, Vec<Box<AST>>, Box<AST>),
    Param(String, ParamAttrib, Box<AST>),
    /// pattern, ty, flag, init: Box<AST>
    Let(Box<AST>, LocalAttrib, Box<AST>, Box<AST>),

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
    OpCast(Box<AST>, Box<AST>),
    /// f: Box<Expr>, ps: Vec<Expr>
    OpCall(Box<AST>, Vec<Box<AST>>),
    /// {id: expr}
    StructExprField(String, Box<AST>),
    /// ty, struct-inits
    OpNew(Box<AST>, Vec<Box<AST>>),

    Id(String),
    TuplePattern(Vec<Box<AST>>),

    /// Type
    TypeBool,
    TypeChar,
    TypeI32,
    TypeF64,
    TypeTuple(Vec<Box<AST>>),
    /// type, dim
    TypeArr(Box<AST>, Box<AST>),
    /// class names
    Path(ModPath),

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
