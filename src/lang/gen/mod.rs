mod builder;
mod gen;
mod lval;

pub use self::builder::{Builder, MethodBuilder};
pub use self::gen::gen;

use super::ast::AST;
use super::mod_mgr::{Arg, Class, Locals, Method, ModMgr, Module};
use crate::ir::ty::IrValType;

use std::cell::RefCell;
use std::collections::HashMap;

pub struct CodeGenCtx<'mgr> {
    pub mgr: &'mgr ModMgr,
    pub module: &'mgr Module,
    pub class: &'mgr Class,
    pub method: &'mgr Method,
    pub locals: RefCell<Locals>,
    pub args_map: HashMap<String, Arg>,
    pub method_builder: RefCell<MethodBuilder>,
}

impl<'mgr> CodeGenCtx<'mgr> {
    fn get_ty(&self, ast: &Box<AST>) -> IrValType {
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
        );
    }
}

pub enum ValType {
    LVal(LValType),
    RVal(IrValType),
    Ret(IrValType),
}

pub enum LValType {
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

impl std::fmt::Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LVal(_) => unimplemented!(),
            Self::RVal(rval) => write!(f, "{} (RVal)", rval),
            Self::Ret(retv) => write!(f, "{} (Ret)", retv),
        }
    }
}

impl ValType {
    pub fn expect_rval(self) -> IrValType {
        match self {
            Self::LVal(_) => panic!("Expect rval but found lval"),
            Self::Ret(_) => panic!("Expect rval but found return value"),
            Self::RVal(val) => val,
        }
    }
}
