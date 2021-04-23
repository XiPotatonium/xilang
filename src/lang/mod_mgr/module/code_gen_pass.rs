use super::super::super::ast::AST;
use super::super::super::gen::{gen, CodeGenCtx, MethodBuilder, ValType};
use super::super::{Class, Locals, Method, ModMgr};
use super::Module;

use xir::tok::{to_tok, TokTag};
use xir::util::path::IModPath;
use xir::{Inst, CCTOR_NAME, CTOR_NAME};

use std::cell::RefCell;

// code gen
impl Module {
    fn code_gen_method(&self, c: &ModMgr, class: &Class, m: &Method, body: &Box<AST>) {
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
        let ret = gen(&ctx, body);

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

    pub fn code_gen(&self, mod_mgr: &ModMgr) {
        for class in self.class_asts.iter() {
            if let AST::Class(class) = class.as_ref() {
                // Set class extend
                if !class.extends_or_impls.is_empty() {
                    let mut builder = self.builder.borrow_mut();
                    for p in class.extends_or_impls.iter() {
                        // find base class
                        let (mod_name, class_name) = self.resolve_path(p, mod_mgr, None);

                        let (extends_idx, extends_idx_tag) =
                            builder.add_const_class(&mod_name, &class_name);
                        let mut class_mut = self.classes.get(&class.name).unwrap().borrow_mut();
                        if let Some(_) = class_mut.extends {
                            panic!("Multiple inheritance for class {}", class.name);
                        }
                        class_mut.extends = Some(
                            mod_mgr
                                .mod_tbl
                                .get(&mod_name)
                                .unwrap()
                                .get_class(&class_name)
                                .unwrap(),
                        );
                        builder.set_class_extends(class_mut.idx, extends_idx, extends_idx_tag);
                    }
                }

                let class_ref = self.classes.get(&class.name).unwrap().borrow();
                // gen static init
                match class.cctor.as_ref() {
                    AST::Block(_) => {
                        let m = class_ref.methods.get(CCTOR_NAME).unwrap();
                        self.code_gen_method(mod_mgr, &class_ref, m, &class.cctor);
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
                        for (i, p) in m.ps.iter().enumerate() {
                            method_builder.add_inst_ldarg((i + 1) as u16);
                            method_builder.add_inst(Inst::StFld(to_tok(
                                class_ref.fields.get(&p.id).unwrap().idx,
                                TokTag::Field,
                            )));
                        }
                    }
                    method_builder.add_inst(Inst::Ret);
                    self.builder.borrow_mut().done(
                        &mut method_builder,
                        m.idx,
                        &vec![],
                        mod_mgr.cfg.optim >= 1,
                    );
                }

                for method_ast in class.methods.iter() {
                    if let AST::Method(method) = method_ast.as_ref() {
                        if let AST::None = method.body.as_ref() {
                            // extern function
                        } else {
                            let m = class_ref.methods.get(&method.name).unwrap();
                            self.code_gen_method(mod_mgr, &class_ref, m, &method.body);
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
