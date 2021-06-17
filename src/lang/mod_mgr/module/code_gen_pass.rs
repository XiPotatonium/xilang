use super::super::super::super::XicCfg;
use super::super::super::ast::AST;
use super::super::super::gen::{
    gen, gen_base_ctor, CodeGenCtx, MethodBuilder, RValType, ValExpectation, ValType,
};
use super::super::{Crate, Locals, Method, Type};
use super::ModuleBuildCtx;

use xir::attrib::MethodImplAttribCodeTypeFlag;
use xir::{Inst, CCTOR_NAME, CTOR_NAME};

use std::cell::RefCell;

// code gen
impl ModuleBuildCtx {
    fn code_gen_method(&self, c: &Crate, class: &Type, m: &Method, optim_level: usize) {
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
                let ast = unsafe { ast.as_ref() };
                match ast {
                    AST::Block(_) => gen(&ctx, ast, ValExpectation::RVal), // cctor
                    AST::Ctor(ctor) => {
                        if !class.extends.is_null() {
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
                        gen(&ctx, &ctor.body, ValExpectation::RVal)
                    }
                    AST::Method(method) => gen(&ctx, &method.body, ValExpectation::RVal),
                    _ => unreachable!(),
                }
            }
            None => {
                // default ctor
                if !class.extends.is_null() {
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

        ctx.done(optim_level);
    }

    pub fn code_gen(&self, mod_mgr: &Crate, cfg: &XicCfg) {
        for class in self.class_asts.iter() {
            match class.as_ref() {
                AST::Class(class) => {
                    let class_ref = self.get_module().classes.get(&class.name).unwrap().as_ref();
                    // gen static init
                    match class.cctor.as_ref() {
                        AST::Block(_) => {
                            let ms = class_ref.methods.get(CCTOR_NAME).unwrap();
                            // only 1 cctor
                            self.code_gen_method(mod_mgr, &class_ref, &ms[0], cfg.optim);
                        }
                        AST::None => (),
                        _ => unreachable!("Parser error"),
                    };

                    let ctors = class_ref.methods.get(CTOR_NAME).unwrap();
                    if class.ctors.is_empty() {
                        // gen default ctor
                        assert_eq!(ctors.len(), 1);
                        self.code_gen_method(mod_mgr, &class_ref, &ctors[0], cfg.optim);
                    } else {
                        for ctor in ctors.iter() {
                            if ctor.impl_flag.is_code_ty(MethodImplAttribCodeTypeFlag::IL) {
                                // only code gen IL method
                                self.code_gen_method(mod_mgr, &class_ref, ctor, cfg.optim);
                            }
                        }
                    }

                    for ms in class_ref.methods.values() {
                        for m in ms.iter() {
                            if m.impl_flag.is_code_ty(MethodImplAttribCodeTypeFlag::IL) {
                                // only code gen IL method
                                self.code_gen_method(mod_mgr, &class_ref, m, cfg.optim);
                            }
                        }
                    }
                }
                AST::Struct(class) => {
                    // Same as class
                    let class_ref = self.get_module().classes.get(&class.name).unwrap().as_ref();
                    // gen static init
                    match class.cctor.as_ref() {
                        AST::Block(_) => {
                            let ms = class_ref.methods.get(CCTOR_NAME).unwrap();
                            // only 1 cctor
                            self.code_gen_method(mod_mgr, &class_ref, &ms[0], cfg.optim);
                        }
                        AST::None => (),
                        _ => unreachable!("Parser error"),
                    };

                    let ctors = class_ref.methods.get(CTOR_NAME).unwrap();
                    if class.ctors.is_empty() {
                        // gen default ctor
                        assert_eq!(ctors.len(), 1);
                        self.code_gen_method(mod_mgr, &class_ref, &ctors[0], cfg.optim);
                    } else {
                        for ctor in ctors.iter() {
                            if ctor.impl_flag.is_code_ty(MethodImplAttribCodeTypeFlag::IL) {
                                // only code gen IL method
                                self.code_gen_method(mod_mgr, &class_ref, ctor, cfg.optim);
                            }
                        }
                    }

                    for ms in class_ref.methods.values() {
                        for m in ms.iter() {
                            if m.impl_flag.is_code_ty(MethodImplAttribCodeTypeFlag::IL) {
                                // only code gen IL method
                                self.code_gen_method(mod_mgr, &class_ref, m, cfg.optim);
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
