mod basic_block;
mod builder;
mod il_gen;
// mod interpreter;
mod method_builder;

pub use basic_block::{BasicBlock, LLCursor};
pub use builder::Builder;
pub use il_gen::{gen, gen_base_ctor};
pub use method_builder::MethodBuilder;

use super::ast::ASTType;
use super::mod_mgr::{Crate, Field, Locals, Method, Module, ModuleBuildCtx, Type};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::ptr::NonNull;

pub enum LoopType {
    Loop(RValType),
    For,
}

pub struct LoopCtx {
    pub ty: LoopType,
    pub continue_target: LLCursor<BasicBlock>,
    pub break_target: LLCursor<BasicBlock>,
}

pub struct CodeGenCtx<'c> {
    pub mgr: &'c Crate,
    pub module: &'c ModuleBuildCtx,
    pub class: &'c Type,
    pub method: &'c Method,
    /// map from ps name to ps idx
    pub ps_map: HashMap<String, usize>,
    pub locals: RefCell<Locals>,
    pub method_builder: RefCell<MethodBuilder>,
    pub loop_ctx: RefCell<Vec<LoopCtx>>,
}

impl<'mgr> CodeGenCtx<'mgr> {
    fn get_ty(&self, ast: &ASTType) -> RValType {
        self.module.get_rval_type(ast, self.mgr, self.class)
    }

    pub fn done(&self, optim_level: usize) {
        let local_mut = self.locals.borrow();
        assert_eq!(
            local_mut.sym_tbl.len(),
            0,
            "Symbol table is not empty after generation"
        );

        self.module.builder.borrow_mut().done(
            &mut self.method_builder.borrow_mut(),
            self.method.idx,
            &local_mut.locals,
            optim_level >= 1,
        );
    }
}

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
    Class(NonNull<Type>),
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
    Never,
    String,
    /// module fullname, class name,
    /// byref if type is reference type, byval if type is value type
    Type(NonNull<Type>),
    ByRef(Box<RValType>),
    /// elety
    Array(Box<RValType>),
}

impl RValType {
    pub fn descriptor(&self) -> String {
        format!("{}", self)
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
            | (Self::String, Self::String)
            | (Self::Void, Self::Void) => true,
            (Self::Type(ty), Self::String) | (Self::String, Self::Type(ty)) => unsafe {
                ty.as_ref().modname() == "std" && ty.as_ref().name == "String"
            },
            (Self::Type(ty1), Self::Type(ty2)) => ty1 == ty2,
            (Self::ByRef(ty0), Self::ByRef(ty1)) => ty0 == ty1,
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
            Self::Never => write!(f, "!"),
            Self::String => write!(f, "Ostd/String;"),
            Self::Type(ty) => write!(f, "O{};", unsafe { ty.as_ref() }),
            Self::ByRef(ty) => write!(f, "&{}", ty),
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

    pub fn expect_sym_ref(&self) -> &SymType {
        match self {
            ValType::RVal(_) => panic!("Expect rval but found rval value"),
            ValType::Sym(val) => val,
            ValType::Ret(_) => panic!("Expect rval but found return value"),
        }
    }
}
