mod basic_block;
mod code_gen_helper;
mod gen;

use std::cell::RefCell;
use std::ptr::NonNull;

use ir::Instruction;

use self::basic_block::{BasicBlock, LLCursor};
use self::code_gen_helper::CodeGenHelper;
use self::gen::gen;

use super::super::ast::AST;
use super::super::sym::{Locals, Method, RValType, ValExpectation, ValType};
use super::super::XicCfg;
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

    pub fn code_gen(&mut self, cfg: &XicCfg) {
        let ret = match self.ast {
            Some(ast) => {
                let ast = unsafe { ast.as_ref() };
                match ast {
                    AST::Block(_) => gen(self, ast, ValExpectation::RVal), // cctor
                    AST::Method(method) => gen(self, &method.body, ValExpectation::RVal),
                    _ => unreachable!(),
                }
            }
            None => {
                // default ctor
                unimplemented!();
            }
        };

        let ret_ty = &unsafe { self.sym.as_ref() }.ret;

        // Check type equivalent
        match &ret {
            ValType::RVal(rval_ty) => {
                if rval_ty != ret_ty {
                    panic!("Expect return {} but return {}", ret_ty, rval_ty);
                }
                // Add return instruction
                self.code_gen_helper
                    .borrow_mut()
                    .add_inst(Instruction::Return);
            }
            ValType::Ret(ret_ty) => {
                if ret_ty != ret_ty {
                    panic!("Expect return {} but return {}", ret_ty, ret_ty);
                }
            }
            _ => unreachable!(),
        }

        self.done(cfg.optim >= 1);
    }

    /// Post-Process
    ///
    /// Fill all jump instructions, concat all basic blocks
    ///
    pub fn done(&mut self, fold_br: bool) {
        // store local var info
        let locals = self.locals.borrow();
        assert!(
            locals.sym_tbl.is_empty(),
            "Symbol table is not empty after generation"
        );
        if locals.local_lst.is_empty() {
            // no locals
        } else {
        }

        if fold_br {
            unimplemented!("Fold branch operation is not implemented");
            // ceq, brfalse -> bne
        }

        let mut code_gen_helper = self.code_gen_helper.borrow_mut();
        let mut offset = 0;
        for bb in code_gen_helper.bb.iter_mut() {
            bb.offset = offset;
            offset += bb.size as i32;
        }

        // fill jump instructions
        for bb in code_gen_helper.bb.iter_mut() {
            if let Some(target) = &bb.target {
                let offset = target.as_ref().unwrap().offset - (bb.size as i32 + bb.offset);
                unimplemented!()
            }
        }

        // concat basic blocks
        let mut code: Vec<Instruction> = Vec::new();
        for bb in code_gen_helper.bb.iter_mut() {
            code.append(&mut bb.insts);
        }

        // Add code to class file
    }
}
