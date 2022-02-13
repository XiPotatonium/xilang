mod basic_block;
mod code_gen_helper;

use std::cell::RefCell;
use std::ptr::NonNull;

use self::basic_block::{BasicBlock, LLCursor};
use self::code_gen_helper::CodeGenHelper;

use super::super::ast::AST;
use super::super::sym::{Locals, Method, RValType, ValExpectation, ValType};
use super::super::XiCfg;
use super::StructBuilder;

pub enum LoopType {
    Loop(RValType),
    For,
}

pub struct LoopCtx {
    pub ty: LoopType,
    pub continue_target: LLCursor<BasicBlock>,
    pub break_target: LLCursor<BasicBlock>,
}

pub struct MethodBuilder {
    pub parent: NonNull<StructBuilder>,
    pub sym: NonNull<Method>,
    /// None for default ctor, ASTBlock for cctor, ASTMethod for normal method
    ast: Option<NonNull<AST>>,
    pub code_gen_helper: RefCell<CodeGenHelper>,
    pub locals: RefCell<Locals>,
    pub loop_ctx: RefCell<Vec<LoopCtx>>,
}

impl MethodBuilder {
    pub fn new(
        parent: NonNull<StructBuilder>,
        method: NonNull<Method>,
        ast: Option<NonNull<AST>>,
    ) -> MethodBuilder {
        MethodBuilder {
            parent,
            sym: method,
            ast,
            code_gen_helper: RefCell::new(CodeGenHelper::new()),
            locals: RefCell::new(Locals::new()),
            loop_ctx: RefCell::new(Vec::new()),
        }
    }

    pub fn code_gen(&mut self, _: &XiCfg) {
        unimplemented!()
    }
}
