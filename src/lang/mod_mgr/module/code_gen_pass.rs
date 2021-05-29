use super::super::super::super::XicCfg;
use super::super::super::ast::{ASTClass, ASTMethodAttribFlag, AST};
use super::super::super::gen::{
    gen, gen_base_ctor, CodeGenCtx, MethodBuilder, RValType, ValExpectation, ValType,
};
use super::super::{Crate, Locals, Method, Type};
use super::ModuleBuildCtx;

use xir::attrib::{FieldAttribFlag, MethodAttribFlag, MethodImplAttribCodeTypeFlag};
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

    /// class extends is set in code gen pass because TypeDef in IrFile may not be set in member pass
    pub fn set_class_extends(&self, mod_mgr: &Crate, class: &ASTClass) {
        let mut builder = self.builder.borrow_mut();
        let mut class_mut = self
            .get_module_mut()
            .classes
            .get_mut(&class.name)
            .unwrap()
            .as_mut();
        for p in class.extends_or_impls.iter() {
            // find base class
            let base = self.resolve_user_define_type(p, mod_mgr, None);
            let base_ref = unsafe { base.as_ref() };

            let (extends_idx, extends_idx_tag) =
                builder.add_const_class(base_ref.modname(), &base_ref.name);
            if !class_mut.extends.is_null() {
                panic!("Multiple inheritance for class {}", class.name);
            }
            class_mut.extends = base.as_ptr() as *const Type;
            builder.set_class_extends(class_mut.idx, extends_idx, extends_idx_tag);
        }

        if class_mut.extends.is_null() {
            // no explicitly designated base class
            // implicitly derived from std::Object
            let class_fullname = format!("{}", class_mut);
            if class_fullname != "std::Object" {
                class_mut.extends = mod_mgr
                    .mod_tbl
                    .get("std")
                    .unwrap()
                    .classes
                    .get("Object")
                    .unwrap()
                    .as_ref() as *const Type;
                let (extends_idx, extends_idx_tag) = builder.add_const_class("std", "Object");
                builder.set_class_extends(class_mut.idx, extends_idx, extends_idx_tag);
            }
        } else {
            // has base class, check extends
            for (field_name, _) in class_mut
                .fields
                .iter()
                .filter(|(_, f)| !f.attrib.is(FieldAttribFlag::Static))
            {
                let mut base = class_mut.extends;
                while let Some(base_ref) = unsafe { base.as_ref() } {
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
                    let method_ast = method.ast.unwrap();
                    let method_ast = if let AST::Method(method_ast) = unsafe { method_ast.as_ref() }
                    {
                        method_ast
                    } else {
                        unreachable!()
                    };
                    let mut has_override = false;
                    let mut base = class_mut.extends;
                    while let Some(base_ref) = unsafe { base.as_ref() } {
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

    pub fn code_gen(&self, mod_mgr: &Crate, cfg: &XicCfg) {
        for class in self.class_asts.iter() {
            match class.as_ref() {
                AST::Class(class) => {
                    self.set_class_extends(mod_mgr, class);

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
                    {
                        let class_mut = self
                            .get_module_mut()
                            .classes
                            .get_mut(&class.name)
                            .unwrap()
                            .as_mut();
                        class_mut.extends = mod_mgr
                            .mod_tbl
                            .get("std")
                            .unwrap()
                            .classes
                            .get("ValueType")
                            .unwrap()
                            .as_ref() as *const Type;
                        let mut builder = self.builder.borrow_mut();
                        let (extends_idx, extends_idx_tag) =
                            builder.add_const_class("std", "ValueType");
                        builder.set_class_extends(class_mut.idx, extends_idx, extends_idx_tag);
                    }

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
