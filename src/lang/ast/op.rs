pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LogAnd,
    LogOr,
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

pub enum UnOp {
    Pos,
    Neg,
    LogNot,
}

pub enum AssignOp {
    AssignEq,
}