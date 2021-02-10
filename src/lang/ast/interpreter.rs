use super::ast::AST;

impl AST {
    pub fn is_const(&self) -> bool {
        match self {
            Self::File(_)
            | Self::Class(_, _, _, _, _)
            | Self::Func(_, _, _, _, _)
            | Self::Field(_, _, _)
            | Self::Param(_, _, _)
            | Self::Let(_, _, _, _)
            | Self::Loop(_)
            | Self::Return(_)
            | Self::Continue
            | Self::Break(_)
            | Self::OpCall(_, _)
            | Self::Id(_)
            | Self::TuplePattern(_)
            | Self::TypeBool
            | Self::TypeChar
            | Self::TypeI32
            | Self::TypeF64
            | Self::TypeTuple(_)
            | Self::TypeClass(_)
            | Self::TypeArr(_, _) => false,
            Self::Block(stmts) => stmts.len() == 0 || (stmts.len() == 1 && stmts[0].is_const()),
            Self::If(cond, then, els) => cond.is_const() && then.is_const() && els.is_const(),
            Self::OpPos(o) => o.is_const(),
            Self::OpNeg(o) => o.is_const(),
            Self::OpAdd(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpSub(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpMul(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpDiv(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpMod(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpLogNot(o1) => o1.is_const(),
            Self::OpLogAnd(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpLogOr(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpEq(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpNe(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpGe(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpGt(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpLe(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpLt(o1, o2) => o1.is_const() && o2.is_const(),
            Self::OpCast(_, v) => v.is_const(),
            Self::StructExprField(_, _) => false,
            Self::OpAssign(_, _)
            | Self::OpNew(_, _)
            | Self::OpStaticAccess(_, _)
            | Self::OpObjAccess(_, _)
            | Self::OpArrayAccess(_, _) => false,
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

pub fn literal_type(ast: &Box<AST>) -> AST {
    match ast.as_ref() {
        AST::Bool(_) => AST::TypeBool,
        AST::Int(_) => AST::TypeI32,
        AST::Float(_) => AST::TypeF64,
        AST::String(_) => unimplemented!(),
        AST::Char(_) => AST::TypeChar,
        AST::Null => panic!("Type of null is unknown"),
        AST::None => AST::None,
        _ => unreachable!(),
    }
}

pub fn const_collapse(ast: &Box<AST>) -> AST {
    match ast.as_ref() {
        AST::Block(stmts) => {
            if stmts.len() == 0 {
                AST::None
            } else if stmts.len() == 1 {
                const_collapse(&stmts[0])
            } else {
                unreachable!();
            }
        }
        AST::If(cond, then, els) => {
            let cond = const_collapse(cond);
            match cond {
                AST::Bool(true) => const_collapse(then),
                AST::Bool(false) => const_collapse(els),
                _ => panic!("Invalid condition in if statement, neither true nor false"),
            }
        }
        AST::OpCast(ty, val) => match const_collapse(val) {
            AST::Bool(v) => match ty.as_ref() {
                AST::TypeBool => AST::Bool(v),
                _ => panic!("Invalid cast. Bool value cannot be cast to other type"),
            },
            AST::Int(v) => match ty.as_ref() {
                AST::TypeI32 => AST::Int(v),
                AST::TypeF64 => AST::Float(v as f64),
                AST::TypeChar => AST::Char(v as u32),
                _ => panic!("Invalid cast. I32 value can only be cast to i32/f64/char"),
            },
            AST::Float(v) => match ty.as_ref() {
                AST::TypeI32 => AST::Int(v as i32),
                AST::TypeF64 => AST::Float(v),
                _ => panic!("Invalid cast. F64 value can only be cast to i32/f64"),
            },
            AST::Char(v) => match ty.as_ref() {
                AST::TypeChar => AST::Char(v),
                AST::TypeI32 => AST::Int(v as i32),
                _ => panic!("Invalid cast. Char value can only be cast to i32/char"),
            },
            AST::String(_) => panic!("Cannot cast string literal"),
            _ => unimplemented!(),
        },
        AST::Null => AST::Null,
        AST::Bool(v) => AST::Bool(*v),
        AST::Int(v) => AST::Int(*v),
        AST::Float(v) => AST::Float(*v),
        AST::Char(v) => AST::Char(*v),
        AST::String(v) => AST::String(v.clone()),
        AST::None => AST::None,
        _ => unimplemented!(),
    }
}
