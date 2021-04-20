use super::super::super::ast::AST;
use super::super::super::gen::{gen, CodeGenCtx, MethodBuilder, ValType};
use super::super::{Class, Locals, Method, ModMgr};
use super::Module;

use xir::tok::{to_tok, TokTag};
use xir::{Inst, CCTOR_NAME, CTOR_NAME};

use std::cell::RefCell;

// code gen
impl Module {
    fn code_gen_method(&self, c: &ModMgr, class: &Class, m: &Method, block: &Box<AST>) {
        let ctx = CodeGenCtx {
            mgr: c,
            module: self,
            class,
            locals: RefCell::new(Locals::new()),
            method: m,
            ps_map: m
                .ps
                .iter()
                .enumerate()
                .map(|(i, p)| (p.id.clone(), i))
                .collect(),
            method_builder: RefCell::new(MethodBuilder::new()),
            loop_ctx: RefCell::new(vec![]),
        };
        let ret = gen(&ctx, block);

        // Check type equivalent
        match &ret {
            ValType::RVal(rval_ty) => {
                if rval_ty != &m.ret {
                    panic!("Expect return {} but return {}", m.ret, rval_ty);
                }
                // Add return instruction
                ctx.method_builder.borrow_mut().add_inst(Inst::Ret);
            }
            ValType::Ret(ret_ty) => {
                if ret_ty != &m.ret {
                    panic!("Expect return {} but return {}", m.ret, ret_ty);
                }
            }
            _ => unreachable!(),
        }

        ctx.done();
    }

    pub fn code_gen(&self, c: &ModMgr) {
        for class in self.class_asts.iter() {
            if let AST::Class(id, _, _, ast_methods, _, ast_init) = class.as_ref() {
                let class_ref = self.classes.get(id).unwrap().borrow();
                // gen static init
                match ast_init.as_ref() {
                    AST::Block(_) => {
                        let m = class_ref.methods.get(CCTOR_NAME).unwrap();
                        self.code_gen_method(c, &class_ref, m, ast_init);
                    }
                    AST::None => (),
                    _ => unreachable!("Parser error"),
                };

                // gen default creator
                // ldarg.0
                // dup
                // ...
                // dup
                // ldarg.1
                // stfld <field0>
                // ldarg.2
                // stfld <field1>
                // ...
                {
                    let m = class_ref.methods.get(CTOR_NAME).unwrap();
                    let mut method_builder = MethodBuilder::new();
                    if m.ps.len() == 0 {
                        // no field
                    } else {
                        method_builder.add_inst_ldarg(0);
                        for _ in (1..m.ps.len()).into_iter() {
                            method_builder.add_inst(Inst::Dup);
                        }
                        for (i, f_id) in (0..m.ps.len())
                            .into_iter()
                            .zip(class_ref.instance_fields.iter())
                        {
                            method_builder.add_inst_ldarg((i + 1) as u16);
                            method_builder.add_inst(Inst::StFld(to_tok(
                                class_ref.fields.get(f_id).unwrap().idx,
                                TokTag::Field,
                            )));
                        }
                    }
                    method_builder.add_inst(Inst::Ret);
                    self.builder.borrow_mut().done(
                        &mut method_builder,
                        m.idx,
                        &vec![],
                        c.cfg.optim >= 1,
                    );
                }

                for method_ast in ast_methods.iter() {
                    if let AST::Method(id, _, _, _, _, block) = method_ast.as_ref() {
                        if let AST::None = block.as_ref() {
                            // extern function
                        } else {
                            let m = class_ref.methods.get(id).unwrap();
                            self.code_gen_method(c, &class_ref, m, block);
                        }
                    } else {
                        unreachable!("Parser error");
                    }
                }
            } else {
                unreachable!();
            }
        }
    }
}
