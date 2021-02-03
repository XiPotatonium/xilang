#[derive(Copy, Clone, Debug)]
pub enum TokenTag {
    // Type
    KwBool,
    KwI32,
    KwF64,
    // Object Oriented
    KwClass,
    KwLSelf,
    KwNew,
    //Crate
    KwSuper,
    KwCrate,
    // Flag
    KwStatic,
    // Other
    KwLet,
    KwLFn,
    KwAs,
    KwFor,
    KwIn,
    KwWhile,
    KwLoop,
    KwIf,
    KwElse,
    KwContinue,
    KwBreak,
    KwReturn,
    // Constant
    KwTrue,
    KwFalse,
    KwNull,
    // Operand
    Id,
    DecLiteral,
    FpLiteral,
    StrLiteral,
    ChLiteral,
    // Arithmetic Operator
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Relation Operator
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    // Logical Operator
    LogAnd,
    LogOr,
    LogNot,
    // Assign Operator
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    // Special Operator
    Arrow,
    DoubleColon,
    Colon,
    Dot,
    LBraces,
    RBraces,
    LParen,
    RParen,
    SemiColon,
    Comma,
    LBracket,
    RBracket,
}

#[derive(Debug)]
pub struct Token {
    pub tag: TokenTag,
    pub literal: Option<String>,
}

impl Token {
    pub fn new(tag: TokenTag, literal: Option<String>) -> Token {
        Token { tag, literal }
    }
}
