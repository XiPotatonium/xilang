use super::super::ast::AST;
use super::{gen, CodeGenCtx, RValType, ValType};

use xir::flag::*;
use xir::path::{IModPath, ModPath};

use std::rc::Weak;

/// Similar to Module.get_ty
pub fn gen_path_lval(ctx: &CodeGenCtx, path: &ModPath, expect_method: bool) -> ValType {
    let (has_crate, super_cnt, path) = path.canonicalize();

    let (mut p, mut segs) = if has_crate {
        let mut p = ModPath::new();
        p.push(ctx.mgr.root.name());
        (p, path.iter().skip(1))
    } else if super_cnt != 0 {
        let mut p = ctx.module.mod_path.as_slice();
        for _ in (0..super_cnt).into_iter() {
            p.to_super();
        }
        (p.to_owned(), path.iter().skip(super_cnt))
    } else {
        let r = path.get_root_name().unwrap();
        if let Some(p) = ctx.module.use_map.get(r) {
            (p.to_owned(), path.iter().skip(1))
        } else if let Some(c) = ctx.module.classes.get(r) {
            if path.len() == 1 {
                return ValType::Class(ctx.module.fullname().to_owned(), r.to_string());
            } else {
                let c = c.borrow();
                let mem = &path[1];
                let ret = if expect_method {
                    if c.methods.contains_key(mem) {
                        ValType::Method(
                            ctx.module.fullname().to_owned(),
                            r.to_owned(),
                            mem.to_owned(),
                        )
                    } else {
                        panic!("No method {} in class {}/{}", mem, ctx.module.fullname(), r);
                    }
                } else if c.fields.contains_key(mem) {
                    ValType::Field(
                        ctx.module.fullname().to_owned(),
                        r.to_owned(),
                        mem.to_owned(),
                    )
                } else {
                    panic!("No field {} in class {}/{}", mem, ctx.module.fullname(), r);
                };
                if path.len() > 2 {
                    panic!("Sub-item in {} is not allowed", ret);
                }
                return ret;
            }
        } else if path.len() == 1 {
            if expect_method {
                if ctx.class.methods.contains_key(r) {
                    return ValType::Method(
                        ctx.module.fullname().to_owned(),
                        ctx.class.name.clone(),
                        r.to_string(),
                    );
                } else {
                    panic!(
                        "No method {} in class {}/{}",
                        r,
                        ctx.module.fullname(),
                        ctx.class.name
                    );
                }
            } else {
                return if ctx.locals.borrow().contains_key(r) {
                    // query local var
                    ValType::Local(r.to_owned())
                } else if ctx.args_map.contains_key(r) {
                    // query args
                    ValType::Arg(r.to_owned())
                } else if ctx.class.fields.contains_key(r) {
                    // query field in this class
                    // either static or non-static is ok
                    ValType::Field(
                        ctx.module.fullname().to_owned(),
                        ctx.class.name.clone(),
                        r.to_owned(),
                    )
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
            let m = Weak::upgrade(m).unwrap();
            if let Some(c) = m.classes.get(seg) {
                if let Some(mem_seg) = segs.next() {
                    // field/method
                    let c = c.borrow();
                    if expect_method {
                        if c.methods.contains_key(mem_seg) {
                            break ValType::Method(
                                mod_path.to_string(),
                                seg.to_string(),
                                mem_seg.to_string(),
                            );
                        } else {
                            panic!(
                                "No method named {} in class {}/{}",
                                mem_seg,
                                mod_path.as_str(),
                                seg
                            );
                        }
                    } else {
                        if c.fields.contains_key(mem_seg) {
                            break ValType::Field(
                                mod_path.to_string(),
                                seg.to_string(),
                                mem_seg.to_string(),
                            );
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
                    break ValType::Class(mod_path.to_string(), seg.to_string());
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
            let lhs = Weak::upgrade(&lhs).unwrap();
            if lhs.classes.contains_key(rhs) {
                ValType::Class(name, rhs.to_owned())
            } else if lhs.sub_mods.contains_key(rhs) {
                ValType::Module(format!("{}/{}", lhs.fullname(), rhs))
            } else {
                panic!("No item {} in module {}", rhs, name);
            }
        }
        ValType::Class(mod_name, name) => {
            // Access a static method or static field in class
            let mod_rc = ctx.mgr.mod_tbl.get(&mod_name).unwrap().upgrade().unwrap();
            let class_ref = mod_rc.classes.get(&name).unwrap().borrow();
            if expect_method {
                if let Some(m) = class_ref.methods.get(rhs) {
                    if m.flag.is(MethodFlagTag::Static) {
                        ValType::Method(mod_name, name, rhs.to_owned())
                    } else {
                        panic!("Cannot static access non-static method {}.{}", name, rhs);
                    }
                } else {
                    panic!("No method {} found in class {}", rhs, name);
                }
            } else {
                if let Some(f) = class_ref.fields.get(rhs) {
                    if f.flag.is(FieldFlagTag::Static) {
                        ValType::Field(mod_name, name, rhs.to_owned())
                    } else {
                        panic!("Cannot static access non-static filed {}.{}", name, rhs);
                    }
                } else {
                    panic!("No field {} found in class {}", rhs, name);
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
                    let mod_rc = ctx.mgr.mod_tbl.get(&mod_name).unwrap().upgrade().unwrap();
                    let class_ref = mod_rc.classes.get(&name).unwrap().borrow();
                    if expect_method {
                        if let Some(m) = class_ref.methods.get(rhs) {
                            if m.flag.is(MethodFlagTag::Static) {
                                panic!("Cannot obj access static method {}::{}", name, rhs);
                            } else {
                                ValType::Method(mod_name, name, rhs.to_owned())
                            }
                        } else {
                            panic!("No method {} found in class {}", rhs, name);
                        }
                    } else {
                        if let Some(f) = class_ref.fields.get(rhs) {
                            if f.flag.is(FieldFlagTag::Static) {
                                panic!("Cannot obj access static filed {}::{}", name, rhs);
                            } else {
                                ValType::Field(mod_name, name, rhs.to_owned())
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
