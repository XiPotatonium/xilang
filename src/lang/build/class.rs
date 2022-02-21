use std::collections::HashMap;
use std::ptr::NonNull;

use super::super::ast::{ASTClass, AST};
use super::super::sym::{Class, TypeLinkContext};
use super::super::XiCfg;
use super::{FieldBuilder, FuncBuilder, ModuleBuilder};
use core::util::ItemPathBuf;

pub struct ClassBuilder {
    pub sym: NonNull<Class>,
    pub methods: Vec<Box<FuncBuilder>>,
    pub fields: Vec<Box<FieldBuilder>>,
    impl_ast: Vec<ItemPathBuf>,
}

impl ClassBuilder {
    pub fn load(path: ItemPathBuf, parent: &mut ModuleBuilder, ast: ASTClass) -> Box<Class> {
        let ASTClass {
            name,
            flags,
            custom_attribs: _,
            impls,
            fields,
            methods,
        } = ast;
        let mut class_sym = Box::new(Class {
            parent: parent.sym,
            path,
            flags,
            impls: Vec::new(),
            fields: HashMap::new(),
            methods: HashMap::new(),
        });
        let mut class_builder = Box::new(ClassBuilder {
            sym: NonNull::new(class_sym.as_ref() as *const Class as *mut Class).unwrap(),
            fields: Vec::new(),
            methods: Vec::new(),
            impl_ast: impls,
        });

        for field in fields.into_iter() {
            if let AST::Field(field_ast) = *field {
                let field_name = field_ast.name.clone();
                if class_sym.fields.contains_key(&field_name) {
                    // a member field and a member method can have the same name
                    panic!(
                        "Already exists a field named {} in class {}",
                        name, class_sym
                    );
                }
                let mut field_path = class_sym.path.clone();
                field_path.push(&field_name);
                class_sym.fields.insert(
                    field_name,
                    FieldBuilder::load(field_path, &mut class_builder, field_ast),
                );
            } else {
                unreachable!()
            }
        }

        for method in methods.into_iter() {
            if let AST::Func(method_ast) = *method {
                let method_name = method_ast.name.clone();
                if class_sym.methods.contains_key(&method_name) {
                    // overload is not allowed
                    // a member field and a member method can have the same name
                    panic!(
                        "Already exists a method named {} in class {}",
                        name, class_sym
                    );
                }
                let mut method_path = class_sym.path.clone();
                method_path.push(&method_name);
                class_sym.methods.insert(
                    method_name,
                    FuncBuilder::load_method(method_path, class_builder.as_mut(), method_ast),
                );
            } else {
                unreachable!()
            }
        }

        parent.classes.push(class_builder);
        class_sym
    }

    pub fn link_type(&mut self, ctx: &TypeLinkContext) {
        for _ in self.impl_ast.iter() {
            unimplemented!()
        }

        for method in self.methods.iter_mut() {
            method.link_type(ctx);
        }

        for field in self.fields.iter_mut() {
            field.link_type(ctx);
        }
    }

    pub fn code_gen(&mut self, cfg: &XiCfg) {
        for method_builder in self.methods.iter_mut() {
            method_builder.code_gen(cfg);
        }
    }
}
