use std::{fmt, ptr::NonNull};

use super::{Class, Symbol};

pub enum ValType {
    /// value on eval stack
    RVal(RValType),
    Sym(Symbol),
    Ret(RValType),
}

#[derive(Clone, Copy)]
pub enum ValExpectation {
    /// return val should be a SymType which is callable
    Callable,
    /// Return val should be a RValType, notice that Empty is allowed,
    /// value-type will only be loaded by value.
    RVal,
    /// Return val shoule be a RValType, Empty is not allowed,
    /// value-type will be loaded byref in preference
    Instance,
    /// return val should be a Module or Class symbol (which may have static member)
    Static,
    /// return lval symbol
    Assignable,
    /// Nothing
    None,
}

#[derive(Clone, Eq)]
pub enum RValType {
    Bool,
    U8,
    Char,
    I32,
    F64,
    None,
    ClassRef(NonNull<Class>),
    /// elety
    Array(Box<RValType>),
    /// Unintialized, for un-linked type
    UnInit,
}

impl PartialEq for RValType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool, Self::Bool)
            | (Self::U8, Self::U8)
            | (Self::Char, Self::Char)
            | (Self::I32, Self::I32)
            | (Self::F64, Self::F64)
            | (Self::None, Self::None) => true,
            (Self::ClassRef(ty1), Self::ClassRef(ty2)) => ty1 == ty2,
            (Self::Array(ele_ty0), Self::Array(ele_ty1)) => ele_ty0 == ele_ty1,
            _ => false,
        }
    }
}

impl fmt::Display for RValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "Z"),
            Self::U8 => write!(f, "B"),
            Self::Char => write!(f, "C"),
            Self::I32 => write!(f, "I"),
            Self::F64 => write!(f, "D"),
            Self::None => write!(f, "V"),
            Self::ClassRef(sym) => write!(f, "O{};", unsafe { sym.as_ref() }),
            Self::Array(ty) => write!(f, "[{}", ty),
            RValType::UnInit => unreachable!(),
        }
    }
}

impl fmt::Display for ValType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RVal(rval) => write!(f, "(RVal){}", rval),
            Self::Sym(sym) => sym.fmt(f),
            Self::Ret(retv) => write!(f, "(Ret){}", retv),
        }
    }
}

impl ValType {
    pub fn expect_rval(self) -> RValType {
        match self {
            ValType::RVal(val) => val,
            ValType::Sym(_) => panic!("Expect rval but found sym value"),
            ValType::Ret(_) => panic!("Expect rval but found return value"),
        }
    }

    pub fn expect_rval_ref(&self) -> &RValType {
        match self {
            ValType::RVal(val) => val,
            ValType::Sym(_) => panic!("Expect rval but found sym value"),
            ValType::Ret(_) => panic!("Expect rval but found return value"),
        }
    }

    pub fn expect_sym(self) -> Symbol {
        match self {
            ValType::RVal(_) => panic!("Expect rval but found rval value"),
            ValType::Sym(val) => val,
            ValType::Ret(_) => panic!("Expect rval but found return value"),
        }
    }
}
