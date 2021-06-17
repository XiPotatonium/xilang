use super::super::super::ast::{ASTType, AST};
use super::super::super::mod_mgr::Method;
use super::super::{CodeGenCtx, RValType, SymType, ValExpectation};
use super::gen;

use xir::attrib::MethodAttribFlag;
use xir::tok::to_tok;
use xir::{Inst, CTOR_NAME};

use std::ptr::{self, NonNull};

pub fn pick_method_from_ptrs(
    candidates: &Vec<NonNull<Method>>,
    args_ty: &Vec<RValType>,
) -> *const Method {
    for candidate in candidates.iter() {
        let candidate_ref = unsafe { candidate.as_ref() };
        if candidate_ref.ps.len() == args_ty.len() {
            let mut is_match = true;
            for (param, arg_ty) in candidate_ref.ps.iter().zip(args_ty.iter()) {
                if &param.ty != arg_ty {
                    is_match = false;
                    break;
                }
            }
            if is_match {
                return candidate.clone().as_ptr();
            }
        }
    }
    ptr::null()
}

pub fn pick_method_from_refs<'m>(
    candidates: &'m Vec<Box<Method>>,
    args_ty: &Vec<RValType>,
) -> Option<&'m Method> {
    for candidate in candidates.iter() {
        if candidate.ps.len() == args_ty.len() {
            let mut is_match = true;
            for (param, arg_ty) in candidate.ps.iter().zip(args_ty.iter()) {
                if &param.ty != arg_ty {
                    is_match = false;
                    break;
                }
            }
            if is_match {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn gen_call(ctx: &CodeGenCtx, f: &Box<AST>, args: &Vec<Box<AST>>) -> RValType {
    let lval = gen(ctx, f, ValExpectation::Callable).expect_sym();
    let (inst, ret) = match &lval {
        SymType::Method(candidates) => {
            // build args
            let args_ty: Vec<RValType> = args
                .iter()
                .map(|arg| gen(ctx, arg, ValExpectation::RVal).expect_rval())
                .collect();

            // only type is checked in gen_call
            // accessibility should be checked in class.query_method
            // static/instance should be checked in gen_val

            // Pick method
            let m_ref = pick_method_from_ptrs(candidates, &args_ty);
            let (mod_name, class_name, m_ref) = if let Some(m_ref) = unsafe { m_ref.as_ref() } {
                unsafe {
                    let class_ref = m_ref.parent.as_ref();
                    let module_ref = class_ref.parent.as_ref();
                    (module_ref.fullname(), &class_ref.name, m_ref)
                }
            } else {
                panic!(
                    "No matched method with param ({}), candidates are: {}",
                    args_ty
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join(", "),
                    candidates
                        .iter()
                        .map(|m| unsafe { m.as_ref().to_string() })
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            };

            // Add to class file
            let sig = ctx.module.builder.borrow_mut().add_method_sig(
                !m_ref.attrib.is(MethodAttribFlag::Static),
                &m_ref.ps,
                &m_ref.ret,
            );
            let (m_idx, tok_tag) = ctx.module.builder.borrow_mut().add_const_member(
                mod_name,
                class_name,
                &m_ref.name,
                sig,
            );

            let mut callvirt = !m_ref.attrib.is(MethodAttribFlag::Static);
            if callvirt {
                let self_ty = unsafe { m_ref.parent.as_ref() };
                if self_ty.is_value_type() {
                    // use call for value type instance methods
                    callvirt = false;
                }
            }

            (
                if callvirt {
                    Inst::CallVirt(to_tok(m_idx, tok_tag))
                } else {
                    Inst::Call(to_tok(m_idx, tok_tag))
                },
                m_ref.ret.clone(),
            )
        }
        SymType::Module(_) => panic!(),
        SymType::Class(_) => panic!(),
        _ => unreachable!(),
    };

    ctx.method_builder.borrow_mut().add_inst(inst);

    ret
}

pub fn gen_new(ctx: &CodeGenCtx, ty: &ASTType, args: &Vec<Box<AST>>) -> RValType {
    let ret = ctx.get_ty(ty);
    let ty = match &ret {
        RValType::Value(ty) => ty,
        RValType::Class(ty) => ty,
        RValType::String => unimplemented!("new string is not implemented"),
        _ => panic!("Cannot new {}", ret),
    };

    let args_ty: Vec<RValType> = args
        .iter()
        .map(|arg| gen(ctx, arg, ValExpectation::RVal).expect_rval())
        .collect();

    let type_ref = unsafe { ty.as_ref() };

    let ctors = type_ref.methods.get(CTOR_NAME).unwrap();

    let ctor = pick_method_from_refs(ctors, &args_ty);
    let ctor = if let Some(ctor) = ctor {
        ctor
    } else {
        panic!("Cannot find ctor");
    };

    let mut builder = ctx.module.builder.borrow_mut();
    let ctor_sig = builder.add_method_sig(true, &ctor.ps, &RValType::Void);
    let (ctor_idx, tok_tag) =
        builder.add_const_member(type_ref.modname(), &type_ref.name, CTOR_NAME, ctor_sig);

    ctx.method_builder
        .borrow_mut()
        .add_inst(Inst::NewObj(to_tok(ctor_idx, tok_tag)));

    ret
}

pub fn gen_new_arr(ctx: &CodeGenCtx, ty: &ASTType, dim: &AST) -> RValType {
    let dim_ty = gen(ctx, dim, ValExpectation::RVal);
    // only i32 or isize if allowed
    match dim_ty.expect_rval_ref() {
        RValType::I32 => {}
        _ => panic!(
            "Array size only support i32 or isize val, but found {}",
            dim_ty
        ),
    }

    let ele_ty = ctx.get_ty(ty);

    let ty_tok = match &ele_ty {
        RValType::Bool | RValType::U8 | RValType::Char | RValType::F64 => unimplemented!(),
        RValType::I32 => {
            let i32_ty = ctx
                .mgr
                .mod_tbl
                .get("std")
                .unwrap()
                .classes
                .get("Int32")
                .unwrap();
            let (idx, tag) = ctx
                .module
                .builder
                .borrow_mut()
                .add_const_class(i32_ty.modname(), &i32_ty.name);
            to_tok(idx, tag.to_tok_tag())
        }
        RValType::String => {
            let str_ty = ctx
                .mgr
                .mod_tbl
                .get("std")
                .unwrap()
                .classes
                .get("String")
                .unwrap();
            let (idx, tag) = ctx
                .module
                .builder
                .borrow_mut()
                .add_const_class(str_ty.modname(), &str_ty.name);
            to_tok(idx, tag.to_tok_tag())
        }
        RValType::Value(ty) | RValType::Class(ty) => {
            let ty_ref = unsafe { ty.as_ref() };
            let (idx, tag) = ctx
                .module
                .builder
                .borrow_mut()
                .add_const_class(ty_ref.modname(), &ty_ref.name);
            to_tok(idx, tag.to_tok_tag())
        }
        _ => {
            unimplemented!("{} array is not implemented", ele_ty)
        }
    };

    ctx.method_builder
        .borrow_mut()
        .add_inst(Inst::NewArr(ty_tok));

    RValType::Array(Box::new(ele_ty))
}
