use core::panic;

use super::super::super::ast::{ASTIdWithGenericParam, AST};
use super::super::super::util::IItemPath;
use super::super::{
    CodeGenCtx, Field, Method, Module, RValType, SymType, Type, ValExpectation, ValType,
};
use super::gen;

use xir::attrib::{FieldAttribFlag, MethodAttribFlag};
use xir::inst::Inst;
use xir::tok::to_tok;

use std::ptr::NonNull;

fn gen_instance_obj_acc(
    ctx: &CodeGenCtx,
    lhs: &Type,
    rhs: &ASTIdWithGenericParam,
    expectation: ValExpectation,
) -> ValType {
    match expectation {
        ValExpectation::None | ValExpectation::Callable => {
            let ms = lhs.query_method(&rhs.id);
            let ms: Vec<NonNull<Method>> = ms
                .into_iter()
                .filter(|m| !m.attrib.is(MethodAttribFlag::Static))
                .map(|m| NonNull::new(m as *const Method as *mut Method).unwrap())
                .collect();
            if ms.is_empty() {
                panic!("No instance method {} found in type {}", rhs, lhs);
            }
            ValType::Sym(SymType::Method(ms))
        }
        ValExpectation::RVal | ValExpectation::Instance => {
            // unlike ValExpectation::Callable,
            // xivm can handle instance field acc of value type correctly, as specified in CLI III.4.10
            if let Some(f) = lhs.query_field(&rhs.id) {
                let field_ty = f.ty.clone();
                let sig = ctx.module.builder.borrow_mut().add_field_sig(&field_ty);
                let (field_idx, tok_tag) = ctx.module.builder.borrow_mut().add_const_member(
                    unsafe { f.parent.as_ref().modname() },
                    unsafe { &f.parent.as_ref().name },
                    &rhs.id,
                    sig,
                );

                let loada = match expectation {
                    ValExpectation::RVal => false,
                    ValExpectation::Instance => {
                        if let RValType::Value(_ty) = field_ty {
                            // load addr if field is a value type
                            true
                        } else {
                            false
                        }
                    }
                    _ => unreachable!(),
                };
                ValType::RVal(if loada {
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst(Inst::LdFldA(to_tok(field_idx, tok_tag)));
                    RValType::ByRef(Box::new(field_ty))
                } else {
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst(Inst::LdFld(to_tok(field_idx, tok_tag)));

                    field_ty
                })
            } else {
                panic!("no field \"{}\" in {}", rhs, lhs);
            }
        }
        ValExpectation::Static => {
            panic!("Type instance member cannot be static accessed")
        }
        ValExpectation::Assignable => {
            if let Some(f) = lhs.query_field(&rhs.id) {
                ValType::Sym(SymType::Field(
                    NonNull::new(f as *const Field as *mut Field).unwrap(),
                ))
            } else {
                panic!("no field \"{}\" in {}", rhs, lhs);
            }
        }
    }
}

pub fn gen_instance_acc(
    ctx: &CodeGenCtx,
    lhs: &AST,
    rhs: &ASTIdWithGenericParam,
    expectation: ValExpectation,
) -> ValType {
    let lhs_ty = gen(ctx, lhs, ValExpectation::Instance).expect_rval();
    match &lhs_ty {
        RValType::String
        | RValType::Class(_)
        | RValType::Value(_)
        | RValType::GenericInst(_, _, _) => {
            let (ty, is_value) = match &lhs_ty {
                RValType::String => (
                    NonNull::new(
                        ctx.mgr
                            .mod_tbl
                            .get("std")
                            .unwrap()
                            .classes
                            .get("String")
                            .unwrap()
                            .as_ref() as *const Type as *mut Type,
                    )
                    .unwrap(),
                    false,
                ),
                RValType::Value(ty) => (ty.clone(), true),
                RValType::Class(ty) => (ty.clone(), false),
                RValType::GenericInst(_, _, _) => todo!(),
                _ => unreachable!(),
            };

            match expectation {
                ValExpectation::None | ValExpectation::Callable => {
                    if is_value {
                        // a value on top of the eval stack, this might be caused by method return
                        // compiler should create a new local var, save value to this local var and then ldloca
                        let loc_idx = ctx.locals.borrow_mut().add_tmp(
                            lhs_ty.clone(),
                            Default::default(),
                            false,
                        );
                        let mut method_builder = ctx.method_builder.borrow_mut();
                        method_builder.add_inst_stloc(loc_idx);
                        method_builder.add_inst_ldloca(loc_idx);
                    }
                }
                _ => {}
            }
            gen_instance_obj_acc(ctx, unsafe { ty.as_ref() }, rhs, expectation)
        }
        RValType::Array(_) => {
            if rhs.id == "len" {
                match expectation {
                    ValExpectation::Callable => {
                        panic!("arr.len is not callable");
                    }
                    ValExpectation::RVal | ValExpectation::Instance => {
                        ctx.method_builder.borrow_mut().add_inst(Inst::LdLen);
                        ValType::RVal(RValType::I32)
                    }
                    ValExpectation::Static => {
                        panic!("arr.len has no static member");
                    }
                    ValExpectation::Assignable => {
                        panic!("arr.len is not assignable");
                    }
                    ValExpectation::None => {
                        panic!("Expect None value but found arr.len");
                    }
                }
            } else {
                panic!("no field \"{}\" in {}", rhs, lhs_ty);
            }
        }
        RValType::ByRef(ty) => match ty.as_ref() {
            RValType::Value(_ty) => {
                let _ty_ref = unsafe { _ty.as_ref() };
                gen_instance_obj_acc(ctx, _ty_ref, rhs, expectation)
            }
            _ => unimplemented!(),
        },
        _ => panic!("no field \"{}\" in {}", rhs, lhs_ty),
    }
}

pub fn gen_static_acc(
    ctx: &CodeGenCtx,
    lhs: &AST,
    rhs: &ASTIdWithGenericParam,
    expectation: ValExpectation,
) -> ValType {
    let lhs_ty = gen(ctx, lhs, ValExpectation::Static).expect_sym();
    match &lhs_ty {
        SymType::Module(m) => match expectation {
            ValExpectation::Callable => {
                panic!("Module member is not callable");
            }
            ValExpectation::RVal => {
                panic!("Module member cannot be loaded as rval");
            }
            ValExpectation::Instance => {
                panic!("Module member cannot be instance accessed");
            }
            ValExpectation::Static => {
                // Access a class or sub-module in module
                let m = unsafe { m.as_ref() };
                if let Some(c) = m.classes.get(&rhs.id) {
                    ValType::Sym(SymType::Class(
                        NonNull::new(c.as_ref() as *const Type as *mut Type).unwrap(),
                    ))
                } else if m.sub_mods.contains(&rhs.id) {
                    let mut path = m.mod_path.clone();
                    path.push(&rhs.id);
                    ValType::Sym(SymType::Module(
                        NonNull::new(ctx.mgr.mod_tbl.get(path.as_str()).unwrap().as_ref()
                            as *const Module as *mut Module)
                        .unwrap(),
                    ))
                } else {
                    panic!("No item {} in module {}", rhs, m);
                }
            }
            ValExpectation::Assignable => {
                panic!("Module member cannot be assigned");
            }
            ValExpectation::None => {
                panic!("Expect None type but found module member");
            }
        },
        SymType::Class(c) => {
            let c = unsafe { c.as_ref() };
            match expectation {
                ValExpectation::None | ValExpectation::Callable => {
                    let ms = c.query_method(&rhs.id);
                    let ms: Vec<NonNull<Method>> = ms
                        .into_iter()
                        .filter(|m| m.attrib.is(MethodAttribFlag::Static))
                        .map(|m| NonNull::new(m as *const Method as *mut Method).unwrap())
                        .collect();
                    if ms.is_empty() {
                        panic!("No static method {} found in class {}", rhs, c);
                    }
                    ValType::Sym(SymType::Method(ms))
                }
                ValExpectation::RVal | ValExpectation::Instance => {
                    if let Some(f) = c.query_field(&rhs.id) {
                        if !f.attrib.is(FieldAttribFlag::Static) {
                            panic!("Field {} in {} is not static", rhs, lhs_ty);
                        }
                        let field_ty = f.ty.clone();

                        let loada = match expectation {
                            ValExpectation::RVal => false,
                            ValExpectation::Instance => {
                                if let RValType::Value(_) = field_ty {
                                    // load addr if field is a value type
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => unreachable!(),
                        };

                        let sig = ctx.module.builder.borrow_mut().add_field_sig(&field_ty);
                        let (field_idx, tok_tag) =
                            ctx.module.builder.borrow_mut().add_const_member(
                                unsafe { f.parent.as_ref().modname() },
                                unsafe { &f.parent.as_ref().name },
                                &rhs.id,
                                sig,
                            );

                        ValType::RVal(if loada {
                            ctx.method_builder
                                .borrow_mut()
                                .add_inst(Inst::LdSFldA(to_tok(field_idx, tok_tag)));

                            RValType::ByRef(Box::new(field_ty))
                        } else {
                            ctx.method_builder
                                .borrow_mut()
                                .add_inst(Inst::LdSFld(to_tok(field_idx, tok_tag)));

                            field_ty
                        })
                    } else {
                        panic!("No field {} in {}", rhs, lhs_ty);
                    }
                }
                ValExpectation::Assignable => {
                    if let Some(f) = c.query_field(&rhs.id) {
                        if !f.attrib.is(FieldAttribFlag::Static) {
                            panic!("Field {} in {} is not static", rhs, lhs_ty);
                        }
                        ValType::Sym(SymType::Field(
                            NonNull::new(f as *const Field as *mut Field).unwrap(),
                        ))
                    } else {
                        panic!("No field {} in {}", rhs, lhs_ty);
                    }
                }
                ValExpectation::Static => {
                    panic!("Type static member cannot be static accessed");
                }
            }
        }
        SymType::Field(f) => {
            let (mod_name, class_name, field_name, field_ty) = unsafe {
                let f_ref = f.as_ref();
                let class_ref = f_ref.parent.as_ref();
                let module_ref = class_ref.parent.as_ref();
                (
                    module_ref.fullname(),
                    &class_ref.name,
                    &f_ref.name,
                    f_ref.ty.clone(),
                )
            };
            let sig = ctx.module.builder.borrow_mut().add_field_sig(&field_ty);
            let (field_idx, tok_tag) = ctx
                .module
                .builder
                .borrow_mut()
                .add_const_member(mod_name, class_name, field_name, sig);
            ctx.method_builder
                .borrow_mut()
                .add_inst(Inst::LdSFld(to_tok(field_idx, tok_tag)));

            ValType::RVal(field_ty)
        }
        _ => unreachable!(),
    }
}

pub fn gen_arr_acc(ctx: &CodeGenCtx, lhs: &AST, rhs: &AST, expectation: ValExpectation) -> ValType {
    let lhs_ty = gen(ctx, lhs, ValExpectation::RVal).expect_rval();
    if let RValType::Array(ele_ty) = lhs_ty {
        let rhs_val = gen(ctx, rhs, ValExpectation::RVal);
        if let ValType::RVal(RValType::I32) = rhs_val {
            match expectation {
                ValExpectation::RVal => {
                    ctx.method_builder
                        .borrow_mut()
                        .add_ldelem(&ele_ty, &ctx.module.builder);
                    ValType::RVal(ele_ty.as_ref().clone())
                }
                ValExpectation::Instance => {
                    let loada = if let RValType::Value(_) = ele_ty.as_ref() {
                        // load addr if element is a value type
                        true
                    } else {
                        false
                    };
                    if loada {
                        ctx.method_builder
                            .borrow_mut()
                            .add_ldelema(&ele_ty, &ctx.module.builder);
                        ValType::RVal(RValType::ByRef(Box::new(ele_ty.as_ref().clone())))
                    } else {
                        ctx.method_builder
                            .borrow_mut()
                            .add_ldelem(&ele_ty, &ctx.module.builder);
                        ValType::RVal(ele_ty.as_ref().clone())
                    }
                }
                ValExpectation::Assignable => {
                    ValType::Sym(SymType::ArrAcc(ele_ty.as_ref().clone()))
                }
                ValExpectation::Callable => panic!("Array element cannot be directly called"),
                ValExpectation::Static => panic!("Cannot static access array element"),
                ValExpectation::None => panic!("Expect None value but found array element"),
            }
        } else {
            panic!("Array index cannot be {}", rhs_val);
        }
    } else {
        panic!("Cannot array access {}", lhs);
    }
}
