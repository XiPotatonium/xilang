use super::super::super::ast::{ASTType, AST};
use super::super::{Class, CodeGenCtx, Field, Method, RValType, ValType};
use super::gen;

use xir::attrib::*;
use xir::util::path::IModPath;

fn gen_id_lval(ctx: &CodeGenCtx, id: &str, expect_method: bool) -> ValType {
    let id = if id == "Self" { &ctx.class.name } else { id };
    if let Some(p) = ctx.module.use_map.get(id) {
        // item in sub module or any using module
        return ValType::Module(p.to_string());
    } else if let Some(c) = ctx.module.classes.get(id) {
        // class within the same module
        return ValType::Class(&c.borrow() as &Class as *const Class);
    } else if ctx.mgr.mod_tbl.contains_key(id) {
        // external module
        // this crate can be referenced in this case (allow or not?)
        return ValType::Module(id.to_owned());
    }

    if expect_method {
        let ms = ctx.class.query_method(id);
        if ms.is_empty() {
            panic!(
                "No method {} in class {}/{}",
                id,
                ctx.module.fullname(),
                ctx.class.name
            );
        }
        return ValType::Method(ms.into_iter().map(|m| m as *const Method).collect());
    } else {
        let is_instance_method = !ctx.method.attrib.is(MethodAttribFlag::Static);
        return if id == "self" {
            if is_instance_method {
                ValType::KwLSelf
            } else {
                panic!("invalid keyword self in static method");
            }
        } else if let Some(var) = ctx.locals.borrow().get(id) {
            // query local var
            ValType::Local(var.idx as usize)
        } else if let Some(arg) = ctx.ps_map.get(id) {
            // query args
            ValType::Arg(*arg)
        } else if let Some(f) = ctx.class.query_field(id) {
            // query field in this class
            // either static or non-static is ok
            ValType::Field(f as *const Field)
        } else {
            panic!("Cannot found item with id: {}", id);
        };
    }
}

fn gen_static_access(ctx: &CodeGenCtx, lhs: ValType, rhs: &str, expect_method: bool) -> ValType {
    match lhs {
        ValType::Module(name) => {
            // Access a class or sub-module in module
            let lhs = ctx.mgr.mod_tbl.get(&name).unwrap();
            if let Some(c) = lhs.get_class(rhs) {
                ValType::Class(c)
            } else if lhs.contains_sub_mod(rhs) {
                ValType::Module(format!("{}/{}", lhs.fullname(), rhs))
            } else {
                panic!("No item {} in module {}", rhs, name);
            }
        }
        ValType::Class(c) => {
            // Access a static method or static field in class
            let class_ref = unsafe { c.as_ref().unwrap() };
            if expect_method {
                let ms = class_ref.query_method(rhs);
                let ms: Vec<*const Method> = ms
                    .into_iter()
                    .filter(|m| m.attrib.is(MethodAttribFlag::Static))
                    .map(|m| m as *const Method)
                    .collect();
                if ms.is_empty() {
                    panic!("No static method {} found in class {}", rhs, class_ref);
                }
                ValType::Method(ms)
            } else {
                if let Some(f) = class_ref.query_field(rhs) {
                    if f.attrib.is(FieldAttribFlag::Static) {
                        ValType::Field(f as *const Field)
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

fn gen_obj_access(ctx: &CodeGenCtx, lhs: RValType, rhs: &str, expect_method: bool) -> ValType {
    if let RValType::Array(_) = lhs {
        if rhs == "len" {
            if expect_method {
                panic!("arr.len is not callable");
            } else {
                return ValType::ArrLen;
            }
        }
    }

    match &lhs {
        RValType::Obj(mod_name, name) => {
            // Access a non-static method or non-static field in class
            let class = ctx
                .mgr
                .mod_tbl
                .get(mod_name)
                .unwrap()
                .get_class(&name)
                .unwrap();
            let class_ref = unsafe { class.as_ref().unwrap() };
            if expect_method {
                let ms = class_ref.query_method(rhs);
                let ms: Vec<*const Method> = ms
                    .into_iter()
                    .filter(|m| !m.attrib.is(MethodAttribFlag::Static))
                    .map(|m| m as *const Method)
                    .collect();
                if ms.is_empty() {
                    panic!("No instance method {} found in class {}", rhs, name);
                }
                ValType::Method(ms)
            } else {
                if let Some(f) = class_ref.query_field(rhs) {
                    if f.attrib.is(FieldAttribFlag::Static) {
                        panic!("Cannot obj access static filed {}::{}", name, rhs);
                    } else {
                        ValType::Field(f as *const Field)
                    }
                } else {
                    panic!("No field {} found in class {}", rhs, name);
                }
            }
        }
        _ => panic!("Cannot obj access a non-obj value"),
    }
}

pub fn gen_lval(ctx: &CodeGenCtx, ast: &AST, expect_method: bool) -> ValType {
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
                    "Self" => {
                        ValType::Class(&ctx.module.classes.get(&ctx.class.name).unwrap().borrow()
                            as &Class as *const Class)
                    }
                    _ => unreachable!(),
                }
            }
            ASTType::None => panic!(),
        },
        AST::Id(id) => gen_id_lval(ctx, id, expect_method),
        AST::OpObjAccess(lhs, rhs) => {
            // generate lhs as lval (as the first arg in instance method or objectref of putfield)
            let lhs = gen(ctx, lhs).expect_rval();
            gen_obj_access(ctx, lhs, rhs, expect_method)
        }
        AST::OpStaticAccess(lhs, rhs) => {
            let lhs = gen_lval(ctx, lhs, expect_method);
            gen_static_access(ctx, lhs, rhs, expect_method)
        }
        AST::OpArrayAccess(lhs, rhs) => {
            let lhs = gen(ctx, lhs).expect_rval();
            if let RValType::Array(ele_ty) = lhs {
                let rhs = gen(ctx, rhs).expect_rval();
                match &rhs {
                    RValType::I32 => {}
                    _ => panic!("Array index cannot be {}", rhs),
                }
                ValType::ArrAcc(ele_ty.as_ref().clone())
            } else {
                panic!("Cannot array access {}", lhs);
            }
        }
        _ => unimplemented!("umimplemented ast: {}", ast),
    }
}
