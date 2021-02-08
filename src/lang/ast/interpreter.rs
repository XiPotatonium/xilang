use super::ast::{Op, AST};

impl AST {
    pub fn is_const(&self) -> bool {
        match self {
            Self::File(_)
            | Self::Class(_, _, _)
            | Self::Func(_, _, _, _)
            | Self::Field(_, _, _)
            | Self::Param(_, _, _)
            | Self::Let(_, _, _, _)
            | Self::Loop(_)
            | Self::Return(_)
            | Self::Continue
            | Self::Break(_)
            | Self::Call(_, _)
            | Self::Id(_)
            | Self::TuplePattern(_)
            | Self::BoolType
            | Self::I32Type
            | Self::F64Type
            | Self::TupleType(_)
            | Self::ClassType(_)
            | Self::ArrType(_, _) => false,
            Self::Block(stmts) => stmts.len() == 0 || (stmts.len() == 1 && stmts[0].is_const()),
            Self::If(cond, then, els) => cond.is_const() && then.is_const() && els.is_const(),
            Self::Unary(op, o1) => op.allow_const() && o1.is_const(),
            Self::Binary(op, o1, o2) => op.allow_const() && o1.is_const() && o2.is_const(),
            Self::Cast(_, v) => v.is_const(),
            Self::Null
            | Self::Bool(_)
            | Self::Int(_)
            | Self::Float(_)
            | Self::Char(_)
            | Self::String(_)
            | Self::None => true,
        }
    }
}

impl Op {
    fn allow_const(&self) -> bool {
        match self {
            Self::Neg
            | Self::Add
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::Mod
            | Self::LogNot
            | Self::LogAnd
            | Self::LogOr
            | Self::Eq
            | Self::Ne
            | Self::Ge
            | Self::Gt
            | Self::Le
            | Self::Lt => true,
            Self::Assign | Self::New | Self::StaticAccess | Self::ObjAccess | Self::ArrayAccess => {
                false
            }
        }
    }
}

pub fn const_collapse(ast: &Box<AST>) -> Box<AST> {
    unimplemented!();
}
