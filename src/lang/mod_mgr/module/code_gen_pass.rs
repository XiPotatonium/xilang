use super::super::super::ast::AST;
use super::super::super::gen::{gen, gen_base_ctor, CodeGenCtx, MethodBuilder, RValType, ValType};
use super::super::{Class, Locals, Method, ModMgr};
use super::Module;

use xir::attrib::MethodImplAttribCodeTypeFlag;
use xir::{Inst, CCTOR_NAME, CTOR_NAME};

use std::cell::RefCell;

// code gen
impl Module {
    fn code_gen_method(&self, c: &ModMgr, class: &Class, m: &Method) {
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

        let ret = match m.ast {
            Some(ast) => {
                let ast = unsafe { ast.as_ref().unwrap() };
                match ast {
                    AST::Block(_) => gen(&ctx, ast), // cctor
                    AST::Ctor(ctor) => {
                        if class.extends.is_some() {
                            // has base class, call base ctor for each ctor
                            if let Some(base_args) = &ctor.base_args {
                                gen_base_ctor(&ctx, base_args);
                            } else {
                                // call default ctor
                                let base_args = Vec::new();
                                gen_base_ctor(&ctx, &base_args);
                            }
                        } else if ctor.base_args.is_some() {
                            // has no base class but has base args
                            panic!(
                                "{} call base ctor but {} actually has no base class",
                                ctor, class
                            );
                        }
                        gen(&ctx, &ctor.body)
                    }
                    AST::Method(method) => gen(&ctx, &method.body),
                    _ => unreachable!(),
                }
            }
            None => {
                // default ctor
                if class.extends.is_some() {
                    let base_args = Vec::new();
                    gen_base_ctor(&ctx, &base_args);
                }
                ValType::RVal(RValType::Void)
            }
        };

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
                {
                    let mut builder = self.builder.borrow_mut();
                    let mut class_mut = self.classes.get(&class.name).unwrap().borrow_mut();
                    for p in class.extends_or_impls.iter() {
                        // find base class
                        let (mod_name, class_name) = self.resolve_path(p, mod_mgr, None);

                        let (extends_idx, extends_idx_tag) =
                            builder.add_const_class(&mod_name, &class_name);
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

                    if let None = class_mut.extends {
                        // no explicitly designated base class
                        // implicitly derived from std::Object
                        let class_fullname = format!("{}", class_mut);
                        if class_fullname != "std::Object" {
                            class_mut.extends = Some(
                                mod_mgr
                                    .mod_tbl
                                    .get("std")
                                    .unwrap()
                                    .get_class("Object")
                                    .unwrap(),
                            );
                            let (extends_idx, extends_idx_tag) =
                                builder.add_const_class("std", "Object");
                            builder.set_class_extends(class_mut.idx, extends_idx, extends_idx_tag);
                        }
                    }
                }

                let class_ref = self.classes.get(&class.name).unwrap().borrow();
                // gen static init
                match class.cctor.as_ref() {
                    AST::Block(_) => {
                        let ms = class_ref.methods.get(CCTOR_NAME).unwrap();
                        // only 1 cctor
                        self.code_gen_method(mod_mgr, &class_ref, &ms[0]);
                    }
                    AST::None => (),
                    _ => unreachable!("Parser error"),
                };

                let ctors = class_ref.methods.get(CTOR_NAME).unwrap();
                if class.ctors.is_empty() {
                    // gen default creator
                    assert_eq!(ctors.len(), 1); // default ctor
                    self.code_gen_method(mod_mgr, &class_ref, &ctors[0]);
                } else {
                    for ctor in ctors.iter() {
                        if ctor.impl_flag.is_code_ty(MethodImplAttribCodeTypeFlag::IL) {
                            // only code gen IL method
                            self.code_gen_method(mod_mgr, &class_ref, ctor);
                        }
                    }
                }

                for ms in class_ref.methods.values() {
                    for m in ms.iter() {
                        if m.impl_flag.is_code_ty(MethodImplAttribCodeTypeFlag::IL) {
                            // only code gen IL method
                            self.code_gen_method(mod_mgr, &class_ref, m);
                        }
                    }
                }
            } else {
                unreachable!();
            }
        }
    }
}
