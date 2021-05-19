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
use super::mod_mgr::{Class, Field, Locals, Method, ModMgr, Module};

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

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
    pub mgr: &'c ModMgr,
    pub module: &'c Module,
    pub class: &'c Class,
    pub method: &'c Method,
    /// map from ps name to ps idx
    pub ps_map: HashMap<String, usize>,
    pub locals: RefCell<Locals>,
    pub method_builder: RefCell<MethodBuilder>,
    pub loop_ctx: RefCell<Vec<LoopCtx>>,
}

impl<'mgr> CodeGenCtx<'mgr> {
    fn get_ty(&self, ast: &ASTType) -> RValType {
        self.module.get_ty(ast, self.mgr, self.class)
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
            self.method.idx,
            &local_mut.locals,
            self.mgr.cfg.optim >= 1,
        );
    }
}

pub enum ValType {
    RVal(RValType),
    Ret(RValType),

    Method(Vec<*const Method>),
    Field(*const Field),
    Class(*const Class),
    // mod fullname
    Module(String),
    // index into locals
    Local(usize),
    // self
    KwLSelf,
    // index into method.ps
    Arg(usize),

    /// arrlen is not handled like a field
    ArrLen,
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
    // module fullname, class name
    Obj(String, String),
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
            (Self::Obj(mod_name, class_name), Self::String)
            | (Self::String, Self::Obj(mod_name, class_name)) => {
                mod_name == "std" && class_name == "String"
            }
            (Self::Obj(mod0, class0), Self::Obj(mod1, class1)) => mod0 == mod1 && class0 == class1,
            (Self::Array(ele_ty1), Self::Array(ele_ty2)) => ele_ty1 == ele_ty2,
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
            Self::Obj(m, s) => write!(f, "O{}/{};", m, s),
            Self::Array(ty) => write!(f, "[{}", ty),
        }
    }
}

impl fmt::Display for ValType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RVal(rval) => write!(f, "(RVal){}", rval),
            Self::Ret(retv) => write!(f, "(Ret){}", retv),
            Self::Method(method) => write!(f, "(Method){}", unsafe { method[0].as_ref().unwrap() }),
            Self::Field(field) => write!(f, "(Field){}", unsafe { field.as_ref().unwrap() }),
            Self::Class(class) => write!(f, "(Class){}", unsafe { class.as_ref().unwrap() }),
            Self::Module(m) => write!(f, "(Mod){}", m),
            Self::Local(n) => write!(f, "(Local){}", n),
            Self::KwLSelf => write!(f, "(Arg)self"),
            Self::Arg(n) => write!(f, "(Arg){}", n),
            Self::ArrLen => write!(f, "(arr.len)"),
            Self::ArrAcc(ele_ty) => write!(f, "(acc){}[]", ele_ty),
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

    pub fn expect_rval_ref(&self) -> &RValType {
        match self {
            Self::Ret(_) => panic!("Expect rval but found return value"),
            Self::RVal(val) => val,
            _ => panic!("Expect rval but found lval"),
        }
    }
}
