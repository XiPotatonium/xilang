use std::{fmt, ptr::NonNull};

use super::{Field, Method, Module, Struct};

pub enum ValType {
    /// value on eval stack
    RVal(RValType),
    Sym(SymType),
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

pub enum SymType {
    Method(Vec<NonNull<Method>>),
    Field(NonNull<Field>),
    Class(NonNull<Struct>),
    Module(NonNull<Module>),
    // index into locals
    Local(usize),
    // self
    KwLSelf,
    // index into method.ps
    Arg(usize),
    /// array element type
    ArrAcc(RValType),
}

#[derive(Clone, Eq)]
pub enum RValType {
    Bool,
    U8,
    Char,
    I32,
    F64,
    Void,
    StructRef(NonNull<Struct>),
    /// elety
    Array(Box<RValType>),
}

impl RValType {
    pub fn stack_size(&self) -> usize {
        match self {
            RValType::Bool | RValType::U8 | RValType::Char | RValType::I32 => 1,
            RValType::F64 => 2,
            RValType::Void => 0,
            RValType::StructRef(_) | RValType::Array(_) => 1,
        }
    }
}

impl PartialEq for RValType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool, Self::Bool)
            | (Self::U8, Self::U8)
            | (Self::Char, Self::Char)
            | (Self::I32, Self::I32)
            | (Self::F64, Self::F64)
            | (Self::Void, Self::Void) => true,
            (Self::StructRef(ty1), Self::StructRef(ty2)) => ty1 == ty2,
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
            Self::Void => write!(f, "V"),
            Self::StructRef(sym) => write!(f, "O{};", unsafe { sym.as_ref() }.path),
            Self::Array(ty) => write!(f, "[{}", ty),
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

impl fmt::Display for SymType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Method(method) => write!(f, "(Method){}", unsafe { method[0].as_ref() }),
            Self::Field(field) => write!(f, "(Field){}", unsafe { field.as_ref() }),
            Self::Class(class) => write!(f, "(Class){}", unsafe { class.as_ref() }),
            Self::Module(m) => write!(f, "(Mod){}", unsafe { m.as_ref() }),
            Self::Local(n) => write!(f, "(Local){}", n),
            Self::KwLSelf => write!(f, "(Arg)self"),
            Self::Arg(n) => write!(f, "(Arg){}", n),
            Self::ArrAcc(ele_ty) => write!(f, "(acc){}[]", ele_ty),
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

    pub fn expect_sym(self) -> SymType {
        match self {
            ValType::RVal(_) => panic!("Expect rval but found rval value"),
            ValType::Sym(val) => val,
            ValType::Ret(_) => panic!("Expect rval but found return value"),
        }
    }
}
