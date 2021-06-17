use super::super::super::ast::{ASTMethodAttribFlag, AST};
use super::super::super::gen::RValType;
use super::super::{Crate, Field, Method, Param, Type};
use super::ModuleBuildCtx;

use xir::attrib::{
    FieldAttribFlag, MethodAttrib, MethodAttribFlag, MethodImplAttrib,
    MethodImplAttribCodeTypeFlag, MethodImplAttribManagedFlag, MethodImplInfoFlag, PInvokeAttrib,
    PInvokeAttribCallConvFlag, PInvokeAttribCharsetFlag, TypeAttribFlag,
};
use xir::{CCTOR_NAME, CTOR_NAME};

use std::ptr::NonNull;

impl ModuleBuildCtx {
    /// Set extends in class info.
    /// Extends info must be set before member pass, because we need this to determine value type and reference type
    fn set_extends1(&self, ast: &AST, class: &mut Type, mod_mgr: &Crate) {
        match ast {
            AST::Class(class_ast) => {
                for p in class_ast.extends_or_impls.iter() {
                    // find base class
                    let base = self.resolve_user_define_type(p, mod_mgr, None);
                    let base_ref = unsafe { base.as_ref() };

                    if base_ref.attrib.is(TypeAttribFlag::Sealed) {
                        panic!("Class {} cannot inherit sealed class {}", class, base_ref);
                    }
                    if !class.extends.is_null() {
                        panic!("Multiple inheritance for class {}", class_ast.name);
                    }
                    class.extends = base.as_ptr() as *const Type;
                }

                if class.extends.is_null() {
                    // no explicitly designated base class
                    // implicitly derived from std::Object
                    let class_fullname = format!("{}", class);
                    if class_fullname != "std::Object" {
                        class.extends = mod_mgr
                            .mod_tbl
                            .get("std")
                            .unwrap()
                            .classes
                            .get("Object")
                            .unwrap()
                            .as_ref() as *const Type;
                    }
                }
            }
            AST::Struct(_) => {
                class.extends = mod_mgr
                    .mod_tbl
                    .get("std")
                    .unwrap()
                    .classes
                    .get("ValueType")
                    .unwrap()
                    .as_ref() as *const Type;
            }
            _ => unreachable!(),
        }
    }

    /// declare method according to ast
    fn declare_method(&self, mod_mgr: &Crate, class_mut: &mut Type, ast: Option<&Box<AST>>) {
        let (ast, name, custom_attribs, attrib, ps, ret) = match ast {
            Some(ast) => {
                let ast_ptr = NonNull::new(ast.as_ref() as *const AST as *mut AST);
                match ast.as_ref() {
                    AST::Block(_) => (
                        ast_ptr,
                        CCTOR_NAME,
                        None,
                        MethodAttrib::from(
                            u16::from(MethodAttribFlag::Pub)
                                | u16::from(MethodAttribFlag::Static)
                                | u16::from(MethodAttribFlag::RTSpecialName)
                                | u16::from(MethodAttribFlag::SpecialName),
                        ),
                        None,
                        RValType::Void,
                    ), // cctor
                    AST::Ctor(ctor) => (
                        ast_ptr,
                        CTOR_NAME,
                        Some(&ctor.custom_attribs),
                        ctor.attrib.clone(),
                        Some(&ctor.ps),
                        RValType::Void,
                    ),
                    AST::Method(method) => {
                        let mut attrib = method.attrib.clone();
                        if method.ast_attrib.is(ASTMethodAttribFlag::Override) {
                            // override implies virtual
                            if attrib.is(MethodAttribFlag::Virtual) {
                                panic!("Method {}.{} is marked as override and cannot be marked as virtual", class_mut, method.name);
                            }
                            attrib.set(MethodAttribFlag::Virtual);
                        } else if attrib.is(MethodAttribFlag::Virtual) {
                            // virtual implies new slot
                            attrib.set(MethodAttribFlag::NewSlot);
                        }

                        (
                            ast_ptr,
                            method.name.as_str(),
                            Some(&method.custom_attribs),
                            attrib,
                            Some(&method.ps),
                            self.get_rval_type(&method.ret, mod_mgr, class_mut),
                        )
                    }
                    _ => unreachable!(),
                }
            }
            None => {
                // default ctor
                (
                    None,
                    CTOR_NAME,
                    None,
                    MethodAttrib::from(
                        u16::from(MethodAttribFlag::Pub)
                            | u16::from(MethodAttribFlag::SpecialName)
                            | u16::from(MethodAttribFlag::RTSpecialName),
                    ),
                    None,
                    RValType::Void,
                )
            }
        };

        let ps = if let Some(ps) = ps {
            ps.iter()
                .map(|p| {
                    if let AST::Param(id, attrib, ty) = p.as_ref() {
                        Param {
                            id: id.to_owned(),
                            ty: self.get_rval_type(ty, mod_mgr, class_mut),
                            attrib: attrib.clone(),
                        }
                    } else {
                        unreachable!();
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let mut impl_flag = MethodImplAttrib::new(
            MethodImplAttribCodeTypeFlag::IL,
            MethodImplAttribManagedFlag::Managed,
        );
        if let Some(custom_attribs) = custom_attribs {
            for attr in custom_attribs.iter() {
                if let AST::CustomAttrib(id, args) = attr.as_ref() {
                    if id == "Dllimport" {
                        // TODO: use real attribute object
                        // Currently it's adhoc
                        assert_eq!(args.len(), 1, "Invalid arg for Dllimport attribute");
                        if let AST::String(_) = args[0].as_ref() {
                            impl_flag.set_code_ty(MethodImplAttribCodeTypeFlag::Native);
                            impl_flag.set_managed(MethodImplAttribManagedFlag::Unmanaged);
                        } else {
                            panic!("Invalid arg for Dllimport attribute");
                        }
                    } else if id == "InternalCall" {
                        assert_eq!(args.len(), 0, "Invalid arg for InternalCall attribute");
                        impl_flag.set_impl_info(MethodImplInfoFlag::InternalCall);
                        impl_flag.set_code_ty(MethodImplAttribCodeTypeFlag::Runtime);
                    } else {
                        panic!("Unrecognizable custom attribute {}", id);
                    }
                } else {
                    unreachable!();
                }
            }
        }

        let method_idx = self
            .builder
            .borrow_mut()
            .add_method(name, &ps, &ret, &attrib, &impl_flag);

        let method = Box::new(Method {
            parent: NonNull::new(class_mut as *mut Type).unwrap(),
            name: name.to_owned(),
            ret,
            ps,
            attrib: attrib.clone(),
            impl_flag: impl_flag.clone(),
            idx: method_idx,
            ast,
        });

        if class_mut.methods.contains_key(name) {
            // check duplication
            let methods = class_mut.methods.get_mut(name).unwrap();
            for m in methods.iter() {
                if m.ps.len() != method.ps.len() {
                    continue;
                }
                let mut is_match = false;
                for (p0, p1) in m.ps.iter().zip(method.ps.iter()) {
                    if p0.ty != p1.ty {
                        is_match = false;
                        break;
                    }
                }
                if is_match {
                    panic!("Duplicated method {}", method);
                }
            }
            methods.push(method);
        } else {
            class_mut.methods.insert(name.to_owned(), vec![method]);
        }

        if let Some(custom_attribs) = custom_attribs {
            for (attr_name, args) in custom_attribs.iter().map(|attr| {
                if let AST::CustomAttrib(id, args) = attr.as_ref() {
                    (id, args)
                } else {
                    unreachable!()
                }
            }) {
                if attr_name == "Dllimport" {
                    // TODO: use real attribute object
                    // Currently it's adhoc
                    if let AST::String(v) = args[0].as_ref() {
                        let pinvoke_attrib = PInvokeAttrib::new(
                            PInvokeAttribCharsetFlag::Ansi,
                            PInvokeAttribCallConvFlag::CDecl,
                        );
                        self.builder.borrow_mut().add_extern_fn(
                            v,
                            name,
                            &pinvoke_attrib,
                            method_idx,
                        );
                    } else {
                        unreachable!();
                    }
                } else if attr_name == "InternalCall" {
                } else {
                    unreachable!();
                }
            }
        }
    }

    /// set extends, declare methods and fields
    pub fn class_pass(&self, mod_mgr: &Crate) {
        for class in self.class_asts.iter() {
            match class.as_ref() {
                AST::Class(class_ast) | AST::Struct(class_ast) => {
                    let mut class_mut = self
                        .get_module_mut()
                        .classes
                        .get_mut(&class_ast.name)
                        .unwrap()
                        .as_mut();
                    // class should be declared after member declarations of previous class
                    //      and before member declarations of this class
                    class_mut.idx = self
                        .builder
                        .borrow_mut()
                        .add_class(&class_ast.name, &class_ast.attrib);
                    // Set extends
                    self.set_extends1(class, class_mut, mod_mgr);

                    // declare fields
                    for field in class_ast.fields.iter() {
                        if let AST::Field(id, flag, _, ty) = field.as_ref() {
                            // Field will have default initialization
                            let ty = self.get_rval_type(ty, mod_mgr, &class_mut);

                            // Build Field in class file
                            let idx = self.builder.borrow_mut().add_field(id, &ty, flag);

                            let field = Box::new(Field {
                                parent: NonNull::new(class_mut as *mut Type).unwrap(),
                                name: id.clone(),
                                attrib: *flag,
                                ty,
                                idx,
                            });

                            if let Some(_) = class_mut.fields.insert(id.to_owned(), field) {
                                // TODO: use expect_none once it becomes stable
                                panic!("Dulicated field {} in class {}", id, class_mut.name);
                            }
                        }
                    }

                    // Add static init
                    match class_ast.cctor.as_ref() {
                        AST::Block(_) => {
                            self.declare_method(mod_mgr, &mut class_mut, Some(&class_ast.cctor));
                        }
                        AST::None => (),
                        _ => unreachable!("Parser error"),
                    };

                    if class_ast.ctors.is_empty() {
                        // Add default object creator
                        self.declare_method(mod_mgr, &mut class_mut, None);
                    } else {
                        for ctor_ast in class_ast.ctors.iter() {
                            self.declare_method(mod_mgr, &mut class_mut, Some(ctor_ast));
                        }
                    }

                    for method_ast in class_ast.methods.iter() {
                        self.declare_method(mod_mgr, &mut class_mut, Some(method_ast));
                    }

                    if self.get_module().is_root() && class_ast.name == "Program" {
                        if let Some(ms) = class_mut.methods.get("main") {
                            for m in ms.iter() {
                                if let RValType::Void = m.ret {
                                    if m.ps.len() == 0
                                        && m.attrib.is(MethodAttribFlag::Pub)
                                        && m.attrib.is(MethodAttribFlag::Static)
                                    {
                                        // pub Program::main()
                                        self.builder.borrow_mut().file.mod_tbl[0].entrypoint =
                                            m.idx;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    /// set extends in ir file and do inheritance check
    pub fn set_extends2(&self) {
        for class in self
            .get_module()
            .classes
            .values()
            .filter(|class| !class.extends.is_null())
        {
            // has base class
            let base = class.extends;
            let base_ref = unsafe { base.as_ref().unwrap() };

            // set extends in IrFile
            // extends info IrFile must be set after all classes (base classes) are declared in IrFile
            let mut builder = self.builder.borrow_mut();
            let (extends_idx, extends_idx_tag) =
                builder.add_const_class(base_ref.modname(), &base_ref.name);
            builder.set_class_extends(class.idx, extends_idx, extends_idx_tag);

            // check extends
            for (field_name, _) in class
                .fields
                .iter()
                .filter(|(_, f)| !f.attrib.is(FieldAttribFlag::Static))
            {
                let mut base = class.extends;
                while let Some(base_ref) = unsafe { base.as_ref() } {
                    if base_ref.fields.contains_key(field_name) {
                        println!("Warning: {} has an instance field {} that override field of base type {}", class, field_name, base_ref);
                        break;
                    }

                    base = base_ref.extends;
                }
            }

            for (method_name, method_grp) in class
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
                    let mut base = class.extends;
                    while let Some(base_ref) = unsafe { base.as_ref() } {
                        if let Some(base_method_grp) = base_ref.methods.get(method_name) {
                            for base_method in base_method_grp.iter() {
                                if method.sig_match(base_method) {
                                    has_override = true;
                                    if !method_ast.ast_attrib.is(ASTMethodAttribFlag::Override) {
                                        println!("Warning: {} has a instance method {} that override method of base type {}", class, method_name, base_ref);
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
                        panic!("Method {}.{} is marked override but no suitable method found to override", class, method);
                    }
                }
            }
        }
    }
}
