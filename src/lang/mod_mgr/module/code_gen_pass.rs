use super::super::super::ast::{ASTClass, ASTMethodAttribFlag, AST};
use super::super::super::gen::{gen, gen_base_ctor, CodeGenCtx, MethodBuilder, RValType, ValType};
use super::super::{Class, Locals, Method, ModMgr};
use super::Module;

use xir::attrib::{FieldAttribFlag, MethodAttribFlag, MethodImplAttribCodeTypeFlag};
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

    /// class extends is set in code gen pass because TypeDef in IrFile may not be set in member pass
    pub fn set_class_extends(&self, mod_mgr: &ModMgr, class: &ASTClass) {
        let mut builder = self.builder.borrow_mut();
        let mut class_mut = self.classes.get(&class.name).unwrap().borrow_mut();
        for p in class.extends_or_impls.iter() {
            // find base class
            let (mod_name, class_name) = self.resolve_path(p, mod_mgr, None);

            let (extends_idx, extends_idx_tag) = builder.add_const_class(&mod_name, &class_name);
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
                let (extends_idx, extends_idx_tag) = builder.add_const_class("std", "Object");
                builder.set_class_extends(class_mut.idx, extends_idx, extends_idx_tag);
            }
        }

        if class_mut.extends.is_some() {
            // has base class, check extends
            for (field_name, _) in class_mut
                .fields
                .iter()
                .filter(|(_, f)| !f.attrib.is(FieldAttribFlag::Static))
            {
                let mut base = class_mut.extends;
                while let Some(base_ptr) = base {
                    let base_ref = unsafe { base_ptr.as_ref().unwrap() };

                    if base_ref.fields.contains_key(field_name) {
                        println!("Warning: {} has an instance field {} that override field of base type {}", class_mut, field_name, base_ref);
                        break;
                    }

                    base = base_ref.extends;
                }
            }

            for (method_name, method_grp) in class_mut
                .methods
                .iter()
                .filter(|(name, _)| *name != CCTOR_NAME && *name != CTOR_NAME)
            {
                for method in method_grp
                    .iter()
                    .filter(|m| !m.attrib.is(MethodAttribFlag::Static))
                {
                    let method_ast = if let AST::Method(method_ast) =
                        unsafe { method.ast.unwrap().as_ref().unwrap() }
                    {
                        method_ast
                    } else {
                        unreachable!()
                    };
                    let mut has_override = false;
                    let mut base = class_mut.extends;
                    while let Some(base_ptr) = base {
                        let base_ref = unsafe { base_ptr.as_ref().unwrap() };

                        if let Some(base_method_grp) = base_ref.methods.get(method_name) {
                            for base_method in base_method_grp.iter() {
                                if method.sig_match(base_method) {
                                    has_override = true;
                                    if !method_ast.ast_attrib.is(ASTMethodAttribFlag::Override) {
                                        println!("Warning: {} has a instance method {} that override method of base type {}", class_mut, method_name, base_ref);
                                    }
                                    break;
                                }
                            }
                            if has_override {
                                break;
                            }
                        }

                        base = base_ref.extends;
                    }

                    if method_ast.ast_attrib.is(ASTMethodAttribFlag::Override) && !has_override {
                        panic!("Method {}.{} is marked override but no suitable method found to override", class_mut, method);
                    }
                }
            }
        }
    }

    pub fn code_gen(&self, mod_mgr: &ModMgr) {
        for class in self.class_asts.iter() {
            if let AST::Class(class) = class.as_ref() {
                self.set_class_extends(mod_mgr, class);

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
                    // gen default ctor
                    assert_eq!(ctors.len(), 1);
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
