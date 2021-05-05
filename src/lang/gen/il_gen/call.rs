use super::super::super::ast::{ASTType, AST};
use super::super::super::mod_mgr::Method;
use super::super::{CodeGenCtx, RValType, ValType};
use super::{gen, lval};

use xir::attrib::MethodAttribFlag;
use xir::tok::to_tok;
use xir::{Inst, CTOR_NAME};

pub fn pick_method_from_ptrs(
    candidates: &Vec<*const Method>,
    args_ty: &Vec<RValType>,
) -> Option<*const Method> {
    for candidate in candidates.iter() {
        let candidate_ref = unsafe { candidate.as_ref().unwrap() };
        if candidate_ref.ps.len() == args_ty.len() {
            let mut is_match = true;
            for (param, arg_ty) in candidate_ref.ps.iter().zip(args_ty.iter()) {
                if &param.ty != arg_ty {
                    is_match = false;
                    break;
                }
            }
            if is_match {
                return Some(*candidate);
            }
        }
    }
    None
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
    let lval = lval::gen_lval(ctx, f, true);
    let (inst, ret) = match &lval {
        ValType::Method(candidates) => {
            // build args
            let args_ty: Vec<RValType> =
                args.iter().map(|arg| gen(ctx, arg).expect_rval()).collect();

            // only type is checked in gen_call
            // accessibility should be checked in class.query_method
            // static/instance should be checked in gen_val

            // Pick method
            let m_ref = pick_method_from_ptrs(candidates, &args_ty);
            let (mod_name, class_name, m_ref) = if let Some(m_ref) = m_ref {
                unsafe {
                    let m_ref = m_ref.as_ref().unwrap();
                    let class_ref = m_ref.parent.as_ref().unwrap();
                    let module_ref = class_ref.parent.as_ref().unwrap();
                    (module_ref.fullname(), &class_ref.name, m_ref)
                }
            } else {
                panic!("Cannot find method");
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

            (
                if m_ref.attrib.is(MethodAttribFlag::Static) {
                    Inst::Call(to_tok(m_idx, tok_tag))
                } else {
                    let tok = to_tok(m_idx, tok_tag);
                    Inst::CallVirt(tok)
                },
                m_ref.ret.clone(),
            )
        }
        ValType::Module(_) => panic!(),
        ValType::Class(_) => panic!(),
        _ => unreachable!(),
    };

    ctx.method_builder.borrow_mut().add_inst(inst);

    ret
}

pub fn gen_new(ctx: &CodeGenCtx, ty: &ASTType, args: &Vec<Box<AST>>) -> RValType {
    let ret = ctx.get_ty(ty);
    match &ret {
        RValType::Obj(mod_name, class_name) => {
            let args_ty: Vec<RValType> =
                args.iter().map(|arg| gen(ctx, arg).expect_rval()).collect();

            let class = ctx
                .mgr
                .mod_tbl
                .get(mod_name)
                .unwrap()
                .get_class(class_name)
                .unwrap();
            let class_ref = unsafe { class.as_ref().unwrap() };
            let ctors = class_ref.methods.get(CTOR_NAME).unwrap();

            let ctor = pick_method_from_refs(ctors, &args_ty);
            let ctor = if let Some(ctor) = ctor {
                ctor
            } else {
                panic!("Cannot find ctor");
            };

            let mut builder = ctx.module.builder.borrow_mut();
            let ctor_sig = builder.add_method_sig(true, &ctor.ps, &RValType::Void);
            let (ctor_idx, tok_tag) =
                builder.add_const_member(mod_name, class_name, CTOR_NAME, ctor_sig);

            ctx.method_builder
                .borrow_mut()
                .add_inst(Inst::NewObj(to_tok(ctor_idx, tok_tag)));
        }
        RValType::Array(_) => unimplemented!(),
        RValType::String => unimplemented!("new string is not implemented"),
        _ => panic!("Invalid new expression, only new class or array is allowed"),
    }
    ret
}
