mod ty;
mod op;

pub use core::flags::{ClassFlag, ClassFlags, FieldFlag, FieldFlags, FuncFlag, FuncFlags};
use core::util::ItemPathBuf;
pub use ty::Type;
pub use op::{BinOp, UnOp, AssignOp};

#[derive(Default)]
pub struct File {
    pub uses: Vec<Use>,
    pub interfaces: Vec<Interface>,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub fns: Vec<Fn>,
    pub globals: Vec<Global>,
}

pub struct Use {
    pub path: ItemPathBuf,
    pub id: Option<String>,
}

pub struct Interface {
    pub custom_attribs: Vec<CustomAttrib>,
    pub id: String,
    pub generic_decl: Option<Vec<String>>,
    pub impls: Vec<ItemPathBuf>,

    pub fns: Vec<Fn>,
}

pub struct Struct {
    pub custom_attribs: Vec<CustomAttrib>,
    pub id: String,
    pub generic_decl: Option<Vec<String>>,
    pub impls: Vec<ItemPathBuf>,

    pub fields: Vec<Field>,
    pub fns: Vec<Fn>,
}

pub struct Enum {
    pub custom_attribs: Vec<CustomAttrib>,
    pub id: String,
    pub generic_decl: Option<Vec<String>>,

    pub fns: Vec<Fn>,
    pub fields: Vec<EnumField>,
}

pub struct Fn {
    pub id: String,
    pub generic_decl: Option<Vec<String>>,

    pub attribs: FuncFlags,
    pub custom_attribs: Vec<CustomAttrib>,
    pub ret: Box<Type>,
    pub ps: Vec<Param>,
    pub body: Option<Box<Block>>,
}

pub struct Global {
    pub id: String,
    pub ty: Box<Type>,
    pub value: Box<Expr>,
}

pub struct EnumField {
    pub id: String,
    pub ty: Option<Box<Type>>,
}

pub struct Field {
    pub id: String,
    pub ty: Box<Type>,
}

pub struct CustomAttrib {
    pub id: String,
    pub args: Option<Vec<Box<Expr>>>,
}

pub struct Param {
    pub id: String,
    pub ty: Box<Type>,
}

pub enum PatKind {
    Id(String),
    Tuple(Vec<PatKind>),
}

pub struct Block {
    pub stmts: Vec<Stmt>,
}

pub struct Stmt {
    pub kind: StmtKind,
}

pub enum StmtKind {
    /// e.g. let a = 10;
    Local(Box<Local>),

    Expr(Box<Expr>),

    /// expr with tailing ;
    Semi(Box<Expr>),

    Empty,
}

pub struct Expr {
    pub kind: ExprKind,
}

pub enum ExprKind {
    /// `[1, 2, 3, a]`
    Array(Vec<Box<Expr>>),

    /// `<f> <args>`
    ///
    /// `<f>(<args>)`
    Call(Box<Expr>, Vec<Box<Expr>>),

    /// `(1, 2, 3, a)`
    Tup(Vec<Box<Expr>>),

    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),

    /// Literal
    Lit(Lit),

    /// `<expr>, <ty>`
    ///
    /// `<expr> as <ty>`
    Cast(Box<Expr>, Box<Type>),

    /// `<cond>, <body>, <els>`
    ///
    /// `if <cond> <body> (else <els>)`
    If(Box<Expr>, Box<Block>, Option<Box<Expr>>),
    /// `<cond>, <body>`
    ///
    /// while <cond> <body>
    While(Box<Expr>, Box<Block>),
    /// `<body>`
    ///
    /// `loop <body>`
    Loop(Box<Block>),

    Block(Box<Block>),

    /// `<op> <lhs> <rhs>`
    ///
    /// `<lhs> <op> <rhs>`
    Assign(AssignOp, Box<Expr>, Box<Expr>),

    /// e.g. `obj.foo`
    Field(Box<Expr>, String),

    /// e.g. `arr[2]`
    Index(Box<Expr>, Box<Expr>),

    Underscore,

    Path(ItemPathBuf),

    Break(Option<Box<Expr>>),
    Continue(),
    Ret(Option<Box<Expr>>),

    Struct(Box<StructExpr>),

    /// e.g. `(a + b)`
    Paren(Box<Expr>),

    Err,
}

pub struct StructExpr {
    pub path: ItemPathBuf,
    pub fields: Vec<ExprField>,
    // rest currently not implemented
}

pub struct ExprField {
    pub field: String,
    /// expr
    pub value: Box<Expr>,
}

pub struct Local {
    pub pattern: PatKind,
    pub ty: Option<Type>,
    pub value: Option<Box<Expr>>,
}

pub enum Lit {
    Null,
    Bool(bool),
    Int(u128),
    Float(f64),
    Str(String),
    Char(char),
}
