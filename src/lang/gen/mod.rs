mod basic_block;
mod builder;
mod gen;
mod lval;
mod op;

pub use self::builder::{Builder, MethodBuilder};
pub use self::gen::gen;

use super::mod_mgr::{Arg, Class, Locals, Method, ModMgr, Module};
use super::{ast::AST, XicCfg};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

pub struct CodeGenCtx<'c> {
    pub mgr: &'c ModMgr,
    pub cfg: &'c XicCfg,
    pub module: &'c Module,
    pub class: &'c Class,
    pub method: &'c Method,
    pub locals: RefCell<Locals>,
    pub args_map: HashMap<String, Arg>,
    pub method_builder: RefCell<MethodBuilder>,
}

impl<'mgr> CodeGenCtx<'mgr> {
    fn get_ty(&self, ast: &Box<AST>) -> RValType {
        self.module.get_ty(ast, self.mgr)
    }

    pub fn done(&self) {
        let local_mut = self.locals.borrow();
        assert_eq!(
            local_mut.sym_tbl.len(),
            0,
            "Symbol table is not empty after generation"
        );

        self.module.builder.borrow_mut().done(
            &mut self.method_builder.borrow_mut(),
            self.method.method_idx,
            local_mut.size(),
            self.cfg.optim >= 1,
        );
    }
}

pub enum ValType {
    RVal(RValType),
    Ret(RValType),

    // mod fullname, class name, method name
    Method(String, String, String),
    // mod fullname, class name, field name
    Field(String, String, String),
    // mod fullname, class name
    Class(String, String),
    // mod fullname
    Module(String),
    // local name
    Local(String),
    // Param name
    Arg(String),
}

#[derive(Clone, Eq)]
pub enum RValType {
    Bool,
    U8,
    Char,
    I32,
    F64,
    Void,
    /// mod fullname, class name
    Obj(String, String),
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
            | (Self::Void, Self::Void) => true,
            (Self::Obj(mod0, class0), Self::Obj(mod1, class1)) => mod0 == mod1 && class0 == class1,
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
            Self::Obj(m, s) => write!(f, "L{}/{};", m, s),
            Self::Array(t) => write!(f, "[{}", t),
        }
    }
}

pub fn fn_descriptor(ret_ty: &RValType, ps: &Vec<RValType>) -> String {
    format!(
        "({}){}",
        ps.iter().map(|t| format!("{}", t)).collect::<String>(),
        ret_ty
    )
}

pub enum LValType {}

impl fmt::Display for ValType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RVal(rval) => write!(f, "{} (RVal)", rval),
            Self::Ret(retv) => write!(f, "{} (Ret)", retv),
            _ => unimplemented!(),
        }
    }
}

impl ValType {
    pub fn expect_rval(self) -> RValType {
        match self {
            Self::Ret(_) => panic!("Expect rval but found return value"),
            Self::RVal(val) => val,
            _ => panic!("Expect rval but found lval"),
        }
    }
}
