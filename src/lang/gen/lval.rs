use super::super::ast::AST;
use super::{gen, Class, CodeGenCtx, Field, Method, RValType, ValType};

use xir::attrib::*;
use xir::util::path::{IModPath, ModPath};

/// Similar to Module.get_ty
pub fn gen_path_lval(ctx: &CodeGenCtx, path: &ModPath, expect_method: bool) -> ValType {
    let (has_crate, super_cnt, path) = path.canonicalize();

    let (mut p, mut segs) = if has_crate {
        // crate::...
        let mut p = ModPath::new();
        p.push(&ctx.mgr.cfg.crate_name);
        (p, path.iter().skip(1))
    } else if super_cnt != 0 {
        // super::...
        let mut p = ctx.module.mod_path.as_slice();
        for _ in (0..super_cnt).into_iter() {
            p.to_super();
        }
        (p.to_owned(), path.iter().skip(super_cnt))
    } else {
        let r = {
            let r = path.get_root_name().unwrap();
            if r == "Self" {
                &ctx.class.name
            } else {
                r
            }
        };
        if let Some(p) = ctx.module.use_map.get(r) {
            // item in sub module or any using module
            (p.to_owned(), path.iter().skip(1))
        } else if let Some(c) = ctx.module.classes.get(r) {
            if path.len() == 1 {
                // class within the same module
                return ValType::Class(&c.borrow() as &Class as *const Class);
            } else {
                // fields or method in class within this module
                let c = c.borrow();
                let mem = &path[1];
                let ret = if expect_method {
                    if let Some(ptr) = c.query_method(mem) {
                        ValType::Method(ptr as *const Method)
                    } else {
                        panic!("No method {} in class {}/{}", mem, ctx.module.fullname(), r);
                    }
                } else if let Some(f) = c.query_field(mem) {
                    ValType::Field(f as *const Field)
                } else {
                    panic!("No field {} in class {}/{}", mem, ctx.module.fullname(), r);
                };
                if path.len() > 2 {
                    panic!("Sub-item in {} is not allowed", ret);
                }
                return ret;
            }
        } else if ctx.mgr.mod_tbl.contains_key(r) {
            // external module
            // this crate can be referenced in this case (allow or not?)
            (ModPath::from_str(r), path.iter().skip(1))
        } else if path.len() == 1 {
            if expect_method {
                if let Some(ptr) = ctx.class.query_method(r) {
                    return ValType::Method(ptr as *const Method);
                } else {
                    panic!(
                        "No method {} in class {}/{}",
                        r,
                        ctx.module.fullname(),
                        ctx.class.name
                    );
                }
            } else {
                let is_instance_method = !ctx.method.attrib.is(MethodAttribFlag::Static);
                return if r == "self" {
                    if is_instance_method {
                        ValType::KwLSelf
                    } else {
                        panic!("invalid keyword self in static method");
                    }
                } else if let Some(var) = ctx.locals.borrow().get(r) {
                    // query local var
                    ValType::Local(var.idx as usize)
                } else if let Some(arg) = ctx.ps_map.get(r) {
                    // query args
                    ValType::Arg(*arg)
                } else if let Some(f) = ctx.class.query_field(r) {
                    // query field in this class
                    // either static or non-static is ok
                    ValType::Field(f as *const Field)
                } else {
                    panic!("Cannot found item with path: {}", path);
                };
            }
        } else {
            panic!("Cannot found item with path: {}", path);
        }
    };

    let ret = loop {
        let seg = if let Some(seg) = segs.next() {
            seg
        } else {
            break ValType::Module(p.to_string());
        };
        p.push(seg);
        if !ctx.mgr.mod_tbl.contains_key(p.as_str()) {
            // not a module, check class
            let mod_path = p.get_super();
            let m = ctx.mgr.mod_tbl.get(mod_path.as_str()).unwrap();
            if let Some(c) = m.get_class(seg) {
                let c = unsafe { c.as_ref().unwrap() };
                if let Some(mem_seg) = segs.next() {
                    // field/method
                    if expect_method {
                        if let Some(method_ref) = c.query_method(mem_seg) {
                            break ValType::Method(method_ref as *const Method);
                        } else {
                            panic!(
                                "No method named {} in class {}/{}",
                                mem_seg,
                                mod_path.as_str(),
                                seg
                            );
                        }
                    } else {
                        if let Some(f) = c.query_field(mem_seg) {
                            break ValType::Field(f as *const Field);
                        } else {
                            panic!(
                                "No field named {} in class {}/{}",
                                mem_seg,
                                mod_path.as_str(),
                                seg
                            );
                        }
                    }
                } else {
                    break ValType::Class(c);
                }
            } else {
                panic!("No class named {} in mod {}", seg, m.fullname());
            }
        }
    };

    // TODO expect none
    if let Some(_) = segs.next() {
        panic!(
            "Invalid path {}: sub-item in method or field is not allowed",
            path.as_str()
        );
    }

    ret
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
                if let Some(m) = class_ref.query_method(rhs) {
                    if m.attrib.is(MethodAttribFlag::Static) {
                        ValType::Method(m as *const Method)
                    } else {
                        panic!(
                            "Cannot static access non-static method {}.{}",
                            class_ref, rhs
                        );
                    }
                } else {
                    panic!("No method {} found in class {}", rhs, class_ref);
                }
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

pub fn gen_lval(ctx: &CodeGenCtx, ast: &Box<AST>, expect_method: bool) -> ValType {
    match ast.as_ref() {
        AST::Path(path) => gen_path_lval(ctx, path, expect_method),
        AST::OpObjAccess(lhs, rhs) => {
            // generate lhs as lval (as the first arg in non-static method or objectref of putfield)
            let lhs = gen(ctx, lhs).expect_rval();
            match lhs {
                RValType::Obj(mod_name, name) => {
                    // Access a non-static method or non-static field in class
                    let class = ctx
                        .mgr
                        .mod_tbl
                        .get(&mod_name)
                        .unwrap()
                        .get_class(&name)
                        .unwrap();
                    let class_ref = unsafe { class.as_ref().unwrap() };
                    if expect_method {
                        if let Some(m) = class_ref.query_method(rhs) {
                            if m.attrib.is(MethodAttribFlag::Static) {
                                panic!("Cannot obj access static method {}::{}", name, rhs);
                            } else {
                                ValType::Method(m as *const Method)
                            }
                        } else {
                            panic!("No method {} found in class {}", rhs, name);
                        }
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
        AST::OpStaticAccess(lhs, rhs) => {
            let lhs = gen_lval(ctx, lhs, expect_method);
            gen_static_access(ctx, lhs, rhs, expect_method)
        }
        AST::OpArrayAccess(_, _) => {
            unimplemented!();
        }
        _ => unimplemented!(),
    }
}
