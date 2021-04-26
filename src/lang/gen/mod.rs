mod basic_block;
mod builder;
mod gen;
// mod interpreter;
mod lval;
mod method_builder;
mod op;

pub use basic_block::{BasicBlock, LLCursor};
pub use builder::Builder;
pub use gen::gen;
pub use method_builder::MethodBuilder;

use xir::blob::EleType;
use xir::file::IrFile;
use xir::tok::{get_tok_tag, TokTag};
use xir::ty::ResolutionScope;

use super::ast::AST;
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
    fn get_ty(&self, ast: &Box<AST>) -> RValType {
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

    Method(*const Method),
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
    // module fullname, class name
    Obj(String, String),
    Array(Box<RValType>),
}

impl RValType {
    pub fn descriptor(&self) -> String {
        format!("{}", self)
    }

    pub fn from_ir_ele_ty(ir_ele_ty: &EleType, ctx: &IrFile) -> RValType {
        match ir_ele_ty {
            EleType::Void => RValType::Void,
            EleType::Boolean => RValType::Bool,
            EleType::Char => RValType::Char,
            EleType::I1 => unimplemented!(),
            EleType::U1 => RValType::U8,
            EleType::I2 => unimplemented!(),
            EleType::U2 => unimplemented!(),
            EleType::I4 => RValType::I32,
            EleType::U4 => unimplemented!(),
            EleType::I8 => unimplemented!(),
            EleType::U8 => unimplemented!(),
            EleType::R4 => unimplemented!(),
            EleType::R8 => RValType::F64,
            EleType::ByRef(t) => {
                if let EleType::Class(tok) = t.as_ref() {
                    // tok is TypeRef or TypeDef
                    let (tag, idx) = get_tok_tag(*tok);
                    let idx = idx as usize - 1;
                    match tag {
                        TokTag::TypeDef => RValType::Obj(
                            ctx.mod_name().to_owned(),
                            ctx.get_str(ctx.typedef_tbl[idx].name).to_owned(),
                        ),
                        TokTag::TypeRef => {
                            let (parent_tag, parent_idx) = ctx.typeref_tbl[idx].get_parent();
                            match parent_tag {
                                ResolutionScope::Mod => unreachable!(),
                                ResolutionScope::ModRef => RValType::Obj(
                                    ctx.get_str(ctx.modref_tbl[parent_idx].name).to_owned(),
                                    ctx.get_str(ctx.typeref_tbl[idx].name).to_owned(),
                                ),
                                ResolutionScope::TypeRef => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                } else {
                    unreachable!();
                }
            }
            EleType::Class(_) => unreachable!(),
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
            Self::Never => write!(f, "!"),
            Self::Obj(m, s) => write!(f, "O{}/{};", m, s),
            Self::Array(t) => write!(f, "[{}", t),
        }
    }
}

impl fmt::Display for ValType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RVal(rval) => write!(f, "(RVal){}", rval),
            Self::Ret(retv) => write!(f, "(Ret){}", retv),
            Self::Method(method) => write!(f, "{}", unsafe { method.as_ref().unwrap() }),
            Self::Field(field) => write!(f, "{}", unsafe { field.as_ref().unwrap() }),
            Self::Class(class) => write!(f, "{}", unsafe { class.as_ref().unwrap() }),
            Self::Module(m) => write!(f, "(Mod){}", m),
            Self::Local(n) => write!(f, "(Local){}", n),
            ValType::KwLSelf => write!(f, "(Arg)self"),
            Self::Arg(n) => write!(f, "(Arg){}", n),
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
