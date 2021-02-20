use super::super::ast::AST;
use super::{gen, CodeGenCtx, LValType};
use crate::ir::flag::*;
use crate::ir::ty::IrValType;

pub fn gen_lval(ctx: &CodeGenCtx, ast: &Box<AST>, expect_method: bool) -> LValType {
    match ast.as_ref() {
        AST::Id(id) => {
            if expect_method {
                // query method in this class
                if ctx.class.methods.contains_key(id) {
                    return LValType::Method(
                        ctx.module.fullname().to_owned(),
                        ctx.class.name.clone(),
                        id.to_owned(),
                    );
                }
            } else {
                // query local var
                if ctx.locals.borrow().contains_key(id) {
                    return LValType::Local(id.to_owned());
                } else if ctx.args_map.contains_key(id) {
                    return LValType::Arg(id.to_owned());
                } else if ctx.class.fields.contains_key(id) {
                    // query field in this class
                    // either static or non-static is ok
                    return LValType::Field(
                        ctx.module.fullname().to_owned(),
                        ctx.class.name.clone(),
                        id.to_owned(),
                    );
                }
            }

            // module or class
            if ctx.module.classes.contains_key(id) {
                // a class in current module
                LValType::Class(ctx.module.fullname().to_owned(), id.to_owned())
            } else if ctx.mgr.mod_tbl.contains_key(id) {
                LValType::Module(id.to_owned())
            } else {
                panic!("Cannot find {}", id);
            }
        }
        AST::OpObjAccess(lhs, rhs) => {
            // generate lhs as lval (as the first arg in non-static method or objectref of putfield)
            let lhs = gen(ctx, lhs).expect_rval();
            match lhs {
                IrValType::Obj(mod_name, name) => {
                    // Access a non-static method or non-static field in class
                    let mod_rc = ctx.mgr.mod_tbl.get(&mod_name).unwrap().upgrade().unwrap();
                    let class_ref = mod_rc.classes.get(&name).unwrap().borrow();
                    if expect_method {
                        if let Some(m) = class_ref.methods.get(rhs) {
                            if m.flag.is(MethodFlagTag::Static) {
                                panic!("Cannot obj access static method {}::{}", name, rhs);
                            } else {
                                LValType::Method(mod_name, name, rhs.to_owned())
                            }
                        } else {
                            panic!("No method {} found in class {}", rhs, name);
                        }
                    } else {
                        if let Some(f) = class_ref.fields.get(rhs) {
                            if f.flag.is(FieldFlagTag::Static) {
                                panic!("Cannot obj access static filed {}::{}", name, rhs);
                            } else {
                                LValType::Field(mod_name, name, rhs.to_owned())
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
            match lhs {
                LValType::Module(_) => {
                    // Access a class or sub-class in module
                    unimplemented!();
                }
                LValType::Class(mod_name, name) => {
                    // Access a static method or static field in class
                    let mod_rc = ctx.mgr.mod_tbl.get(&mod_name).unwrap().upgrade().unwrap();
                    let class_ref = mod_rc.classes.get(&name).unwrap().borrow();
                    if expect_method {
                        if let Some(m) = class_ref.methods.get(rhs) {
                            if m.flag.is(MethodFlagTag::Static) {
                                LValType::Method(mod_name, name, rhs.to_owned())
                            } else {
                                panic!("Cannot static access non-static method {}.{}", name, rhs);
                            }
                        } else {
                            panic!("No method {} found in class {}", rhs, name);
                        }
                    } else {
                        if let Some(f) = class_ref.fields.get(rhs) {
                            if f.flag.is(FieldFlagTag::Static) {
                                LValType::Field(mod_name, name, rhs.to_owned())
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
        AST::OpArrayAccess(_, _) => {
            unimplemented!();
        }
        _ => unimplemented!(),
    }
}
