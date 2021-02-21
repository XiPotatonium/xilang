use super::super::ast::AST;
use super::{gen, CodeGenCtx, RValType, ValType};

use crate::ir::flag::*;
use crate::ir::path::{IModPath, ModPathSlice};

use std::rc::Weak;

pub fn gen_path_lval(ctx: &CodeGenCtx, path: ModPathSlice, expect_method: bool) -> ValType {
    if path.len() == 1 {
        let id = path.as_str();
        if expect_method {
            // query method in this class
            if ctx.class.methods.contains_key(id) {
                return ValType::Method(
                    ctx.module.fullname().to_owned(),
                    ctx.class.name.clone(),
                    id.to_owned(),
                );
            }
        } else {
            // query local var
            if ctx.locals.borrow().contains_key(id) {
                return ValType::Local(id.to_owned());
            } else if ctx.args_map.contains_key(id) {
                return ValType::Arg(id.to_owned());
            } else if ctx.class.fields.contains_key(id) {
                // query field in this class
                // either static or non-static is ok
                return ValType::Field(
                    ctx.module.fullname().to_owned(),
                    ctx.class.name.clone(),
                    id.to_owned(),
                );
            }
        }

        // module or class
        if id == "crate" {
            ValType::Module(ctx.mgr.root.name().to_owned())
        } else if ctx.module.classes.contains_key(id) {
            // a class in current module
            ValType::Class(ctx.module.fullname().to_owned(), id.to_owned())
        } else if ctx.mgr.mod_tbl.contains_key(id) {
            ValType::Module(id.to_owned())
        } else {
            panic!("Cannot find {}", id);
        }
    } else {
        let mut path = path;
        // TODO optimize to_owned
        let rhs = path.get_self_name().unwrap().to_owned();
        path.to_super();
        let lhs = gen_path_lval(ctx, path, expect_method);

        gen_static_access(ctx, lhs, &rhs, expect_method)
    }
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
        AST::Path(path) => {
            let (_, _, path) = path.canonicalize();
            gen_path_lval(ctx, path.as_slice(), expect_method)
        }
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
