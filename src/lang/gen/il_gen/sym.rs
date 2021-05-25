use std::ptr::NonNull;

use super::super::super::ast::{ASTType, AST};
use super::super::{Class, CodeGenCtx, Field, Method, RValType, SymType, SymUsage};
use super::gen;

use xir::attrib::*;
use xir::util::path::IModPath;

fn gen_id_sym(ctx: &CodeGenCtx, id: &str, usage: SymUsage) -> SymType {
    let id = if id == "Self" { &ctx.class.name } else { id };
    if let Some(p) = ctx.module.use_map.get(id) {
        // item in sub module or any using module
        return SymType::Module(p.to_string());
    } else if ctx.module.get_module().sub_mods.contains(id) {
        // a submodule in this module
        let mut path = ctx.module.get_module().mod_path.clone();
        path.push(id);
        return SymType::Module(path.to_string());
    } else if let Some(c) = ctx.module.get_module().classes.get(id) {
        // class within the same module
        return SymType::Class(NonNull::new(c.as_ref() as *const Class as *mut Class).unwrap());
    } else if ctx.mgr.mod_tbl.contains_key(id) {
        // external module
        // this crate can be referenced in this case (allow or not?)
        return SymType::Module(id.to_owned());
    }

    if usage.is_callee() {
        let ms = ctx.class.query_method(id);
        if ms.is_empty() {
            panic!(
                "No method {} in class {}/{}",
                id,
                ctx.module.get_module().fullname(),
                ctx.class.name
            );
        }
        return SymType::Method(
            ms.into_iter()
                .map(|m| NonNull::new(m as *const Method as *mut Method).unwrap())
                .collect(),
        );
    } else {
        let is_instance_method = !ctx.method.attrib.is(MethodAttribFlag::Static);
        return if id == "self" {
            if is_instance_method {
                SymType::KwLSelf
            } else {
                panic!("invalid keyword self in static method");
            }
        } else if let Some(var) = ctx.locals.borrow().get(id) {
            // query local var
            SymType::Local(var.idx as usize)
        } else if let Some(arg) = ctx.ps_map.get(id) {
            // query args
            SymType::Arg(*arg)
        } else if let Some(f) = ctx.class.query_field(id) {
            // query field in this class
            // either static or non-static is ok
            SymType::Field(NonNull::new(f as *const Field as *mut Field).unwrap())
        } else {
            panic!("Cannot found item with id: {}", id);
        };
    }
}

fn gen_static_access(ctx: &CodeGenCtx, lhs: SymType, rhs: &str, usage: SymUsage) -> SymType {
    match lhs {
        SymType::Module(name) => {
            // Access a class or sub-module in module
            let lhs = ctx.mgr.mod_tbl.get(&name).unwrap();
            if let Some(c) = lhs.classes.get(rhs) {
                SymType::Class(NonNull::new(c.as_ref() as *const Class as *mut Class).unwrap())
            } else if lhs.sub_mods.contains(rhs) {
                SymType::Module(format!("{}/{}", lhs.fullname(), rhs))
            } else {
                panic!("No item {} in module {}", rhs, name);
            }
        }
        SymType::Class(c) => {
            // Access a static method or static field in class
            let class_ref = unsafe { c.as_ref() };
            if usage.is_callee() {
                let ms = class_ref.query_method(rhs);
                let ms: Vec<NonNull<Method>> = ms
                    .into_iter()
                    .filter(|m| m.attrib.is(MethodAttribFlag::Static))
                    .map(|m| NonNull::new(m as *const Method as *mut Method).unwrap())
                    .collect();
                if ms.is_empty() {
                    panic!("No static method {} found in class {}", rhs, class_ref);
                }
                SymType::Method(ms)
            } else {
                if let Some(f) = class_ref.query_field(rhs) {
                    if f.attrib.is(FieldAttribFlag::Static) {
                        SymType::Field(NonNull::new(f as *const Field as *mut Field).unwrap())
                    } else {
                        panic!(
                            "Cannot static access non-static filed {}.{}",
                            class_ref, rhs
                        );
                    }
                } else {
                    panic!("No field {} found in class {}", rhs, class_ref);
                }
            }
        }
        _ => unimplemented!(),
    }
}

fn _gen_obj_access(
    mod_name: &str,
    name: &str,
    ctx: &CodeGenCtx,
    rhs: &str,
    usage: SymUsage,
) -> SymType {
    // Access a non-static method or non-static field in class
    let class_ref = ctx
        .mgr
        .mod_tbl
        .get(mod_name)
        .unwrap()
        .classes
        .get(name)
        .unwrap()
        .as_ref();
    if usage.is_callee() {
        let ms = class_ref.query_method(rhs);
        let ms: Vec<NonNull<Method>> = ms
            .into_iter()
            .filter(|m| !m.attrib.is(MethodAttribFlag::Static))
            .map(|m| NonNull::new(m as *const Method as *mut Method).unwrap())
            .collect();
        if ms.is_empty() {
            panic!("No instance method {} found in class {}", rhs, name);
        }
        SymType::Method(ms)
    } else {
        if let Some(f) = class_ref.query_field(rhs) {
            if f.attrib.is(FieldAttribFlag::Static) {
                panic!("Cannot obj access static filed {}::{}", name, rhs);
            } else {
                SymType::Field(NonNull::new(f as *const Field as *mut Field).unwrap())
            }
        } else {
            panic!("No field {} found in class {}", rhs, name);
        }
    }
}

fn gen_obj_access(ctx: &CodeGenCtx, lhs: RValType, rhs: &str, usage: SymUsage) -> SymType {
    if let RValType::Array(_) = lhs {
        if rhs == "len" {
            if usage.is_callee() {
                panic!("arr.len is not callable");
            } else {
                return SymType::ArrLen;
            }
        }
    }

    match &lhs {
        RValType::Obj(mod_name, name) => _gen_obj_access(mod_name, name, ctx, rhs, usage),
        RValType::String => _gen_obj_access("std", "String", ctx, rhs, usage),
        _ => panic!("Cannot obj access a non-obj value"),
    }
}

pub fn gen_sym(ctx: &CodeGenCtx, ast: &AST, usage: SymUsage) -> SymType {
    match ast {
        AST::Type(ty) => match ty.as_ref() {
            ASTType::Bool => unimplemented!(),
            ASTType::Char => unimplemented!(),
            ASTType::I32 => unimplemented!(),
            ASTType::F64 => unimplemented!(),
            ASTType::String => unimplemented!(),
            ASTType::Tuple(_) => unimplemented!(),
            ASTType::Arr(_) => unimplemented!(),
            ASTType::Class(path) => {
                assert!(path.len() == 1, "invalid path in lval gen \"{}\"", path);
                match path.as_str() {
                    "Self" => SymType::Class(
                        NonNull::new(
                            ctx.module
                                .get_module()
                                .classes
                                .get(&ctx.class.name)
                                .unwrap()
                                .as_ref() as *const Class as *mut Class,
                        )
                        .unwrap(),
                    ),
                    _ => unreachable!(),
                }
            }
            ASTType::None => panic!(),
        },
        AST::Id(id) => gen_id_sym(ctx, id, usage),
        AST::OpObjAccess(lhs, rhs) => {
            // generate lhs as lval (as the first arg in instance method or objectref of putfield)
            let lhs = gen(ctx, lhs).expect_rval();
            gen_obj_access(ctx, lhs, rhs, usage)
        }
        AST::OpStaticAccess(lhs, rhs) => {
            let lhs = gen_sym(ctx, lhs, usage);
            gen_static_access(ctx, lhs, rhs, usage)
        }
        AST::OpArrayAccess(lhs, rhs) => {
            let lhs = gen(ctx, lhs).expect_rval();
            if let RValType::Array(ele_ty) = lhs {
                let rhs = gen(ctx, rhs).expect_rval();
                match &rhs {
                    RValType::I32 => {}
                    _ => panic!("Array index cannot be {}", rhs),
                }
                SymType::ArrAcc(ele_ty.as_ref().clone())
            } else {
                panic!("Cannot array access {}", lhs);
            }
        }
        _ => unimplemented!("umimplemented ast: {}", ast),
    }
}
