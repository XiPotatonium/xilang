use super::super::super::ast::{ASTMethodAttribFlag, AST};
use super::super::super::gen::RValType;
use super::super::{Class, Field, Method, ModMgr, ModRef, Param};
use super::Module;

use xir::attrib::{
    MethodAttrib, MethodAttribFlag, MethodImplAttrib, MethodImplAttribCodeTypeFlag,
    MethodImplAttribManagedFlag, MethodImplInfoFlag, PInvokeAttrib, PInvokeAttribCallConvFlag,
    PInvokeAttribCharsetFlag,
};
use xir::{CCTOR_NAME, CTOR_NAME};

// member pass
impl Module {
    /// declare method according to ast
    fn declare_method(&self, mod_mgr: &ModMgr, class_mut: &mut Class, ast: Option<&Box<AST>>) {
        let (ast, name, custom_attribs, attrib, ps, ret) = match ast {
            Some(ast) => {
                let ast_ptr = Some(ast.as_ref() as *const AST);
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
                            self.get_ty(&method.ret, mod_mgr, class_mut),
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
                            ty: self.get_ty(ty, mod_mgr, class_mut),
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
            parent: class_mut as &Class as *const Class,
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

    pub fn member_pass(&self, mod_mgr: &ModMgr) {
        for class in self.class_asts.iter() {
            if let AST::Class(class) = class.as_ref() {
                let mut class_mut = self.classes.get(&class.name).unwrap().borrow_mut();
                class_mut.parent =
                    mod_mgr.mod_tbl.get(self.fullname()).unwrap().as_ref() as *const ModRef;
                class_mut.idx = self
                    .builder
                    .borrow_mut()
                    .add_class(&class.name, &class.attrib);

                for field in class.fields.iter() {
                    if let AST::Field(id, flag, _, ty) = field.as_ref() {
                        // Field will have default initialization
                        let ty = self.get_ty(ty, mod_mgr, &class_mut);

                        // Build Field in class file
                        let idx = self.builder.borrow_mut().add_field(id, &ty, flag);

                        let field = Box::new(Field {
                            parent: &class_mut as &Class as *const Class,
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
                match class.cctor.as_ref() {
                    AST::Block(_) => {
                        self.declare_method(mod_mgr, &mut class_mut, Some(&class.cctor));
                    }
                    AST::None => (),
                    _ => unreachable!("Parser error"),
                };

                if class.ctors.is_empty() {
                    // Add default object creator
                    self.declare_method(mod_mgr, &mut class_mut, None);
                } else {
                    for ctor_ast in class.ctors.iter() {
                        self.declare_method(mod_mgr, &mut class_mut, Some(ctor_ast));
                    }
                }

                for method_ast in class.methods.iter() {
                    self.declare_method(mod_mgr, &mut class_mut, Some(method_ast));
                }

                if self.is_root() && class.name == "Program" {
                    if let Some(ms) = class_mut.methods.get("main") {
                        for m in ms.iter() {
                            if let RValType::Void = m.ret {
                                if m.ps.len() == 0
                                    && m.attrib.is(MethodAttribFlag::Pub)
                                    && m.attrib.is(MethodAttribFlag::Static)
                                {
                                    // pub Program::main()
                                    self.builder.borrow_mut().file.mod_tbl[0].entrypoint = m.idx;
                                    break;
                                }
                            }
                        }
                    }
                }
            } else {
                unreachable!();
            }
        }
    }
}
