use super::super::super::ast::AST;
use super::super::super::gen::RValType;
use super::super::{Class, Field, Method, ModMgr, ModRef, Param};
use super::Module;

use xir::attrib::{
    FieldAttribFlag, MethodAttrib, MethodAttribFlag, MethodImplAttrib,
    MethodImplAttribCodeTypeFlag, MethodImplAttribManagedFlag, PInvokeAttrib,
    PInvokeAttribCallConvFlag, PInvokeAttribCharsetFlag, ParamAttrib,
};
use xir::{CCTOR_NAME, CTOR_NAME};

// use macro to avoid borrow mutable self twice, SB rust
macro_rules! declare_method {
    ($class: expr, $builder: expr, $name: expr, $flag: expr, $impl_flag: expr, $ret: expr, $ps: expr) => {{
        let idx = $builder
            .borrow_mut()
            .add_method($name, &$ps, &$ret, $flag, $impl_flag);

        let method = Box::new(Method {
            parent: &$class as &Class as *const Class,
            name: $name.to_owned(),
            ret: $ret,
            ps: $ps,
            attrib: $flag.clone(),
            impl_flag: $impl_flag.clone(),
            idx,
        });
        // let sig = format!("{}{}", $id, method.descriptor());

        if let Some(_) = $class.methods.insert($name.to_owned(), method) {
            // TODO: use expect_none once it becomes stable
            panic!("Duplicated method {} in class {}", $name, $class.name);
        }
        idx
    }};
}

// member pass
impl Module {
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
                        let ret = RValType::Void;
                        let ps: Vec<Param> = vec![];
                        let flag = MethodAttrib::from(
                            u16::from(MethodAttribFlag::Pub)
                                | u16::from(MethodAttribFlag::Static)
                                | u16::from(MethodAttribFlag::RTSpecialName),
                        );
                        let impl_flag = MethodImplAttrib::new(
                            MethodImplAttribCodeTypeFlag::IL,
                            MethodImplAttribManagedFlag::Managed,
                        );
                        declare_method!(
                            class_mut,
                            self.builder,
                            CCTOR_NAME,
                            &flag,
                            &impl_flag,
                            ret,
                            ps
                        );
                    }
                    AST::None => (),
                    _ => unreachable!("Parser error"),
                };

                // Add default object creator
                // TODO: use C# like default ctor
                {
                    let ret = RValType::Void;
                    let ps: Vec<Param> = class_mut
                        .fields
                        .iter()
                        .filter(|(_, f)| !f.attrib.is(FieldAttribFlag::Static))
                        .map(|(id, f)| Param {
                            id: id.to_owned(),
                            attrib: ParamAttrib::default(),
                            ty: f.ty.clone(),
                        })
                        .collect();
                    let flag = MethodAttrib::from(
                        u16::from(MethodAttribFlag::Pub)
                            | u16::from(MethodAttribFlag::RTSpecialName),
                    );
                    let impl_flag = MethodImplAttrib::new(
                        MethodImplAttribCodeTypeFlag::IL,
                        MethodImplAttribManagedFlag::Managed,
                    );
                    declare_method!(
                        class_mut,
                        self.builder,
                        CTOR_NAME,
                        &flag,
                        &impl_flag,
                        ret,
                        ps
                    );
                }

                for method in class.methods.iter() {
                    if let AST::Method(method) = method.as_ref() {
                        let ps = method
                            .ps
                            .iter()
                            .map(|p| {
                                if let AST::Param(id, attrib, ty) = p.as_ref() {
                                    Param {
                                        id: id.to_owned(),
                                        ty: self.get_ty(ty, mod_mgr, &class_mut),
                                        attrib: attrib.clone(),
                                    }
                                } else {
                                    unreachable!();
                                }
                            })
                            .collect();
                        let ret = self.get_ty(&method.ret, mod_mgr, &class_mut);
                        let mut impl_flag = MethodImplAttrib::new(
                            MethodImplAttribCodeTypeFlag::IL,
                            MethodImplAttribManagedFlag::Managed,
                        );
                        for attr in method.custom_attribs.iter() {
                            if let AST::CustomAttrib(id, args) = attr.as_ref() {
                                if id == "Dllimport" {
                                    // TODO: use real attribute object
                                    // Currently it's adhoc
                                    assert_eq!(
                                        args.len(),
                                        1,
                                        "Invalid arg for Dllimport attribute"
                                    );
                                    if let AST::String(_) = args[0].as_ref() {
                                        impl_flag.set_code_ty(MethodImplAttribCodeTypeFlag::Native);
                                        impl_flag
                                            .set_managed(MethodImplAttribManagedFlag::Unmanaged);
                                    } else {
                                        panic!("Invalid arg for Dllimport attribute");
                                    }
                                } else {
                                    panic!("Unrecognizable custom attribute {}", id);
                                }
                            } else {
                                unreachable!();
                            }
                        }

                        let method_idx = declare_method!(
                            class_mut,
                            self.builder,
                            &method.name,
                            &method.attrib,
                            &impl_flag,
                            ret,
                            ps
                        );
                        for (attr_name, args) in method.custom_attribs.iter().map(|attr| {
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
                                        &method.name,
                                        &pinvoke_attrib,
                                        method_idx,
                                    );
                                } else {
                                    unreachable!();
                                }
                            } else {
                                unreachable!();
                            }
                        }
                    }
                }

                if self.is_root() && class.name == "Program" {
                    if let Some(m) = class_mut.methods.get("main") {
                        if let RValType::Void = m.ret {
                            if m.ps.len() == 0
                                && m.attrib.is(MethodAttribFlag::Pub)
                                && m.attrib.is(MethodAttribFlag::Static)
                            {
                                // pub Program::main()
                                self.builder.borrow_mut().file.mod_tbl[0].entrypoint = m.idx;
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
