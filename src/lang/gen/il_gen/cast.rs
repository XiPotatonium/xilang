use super::super::super::ast::{ASTType, AST};
use super::super::super::mod_mgr::Type;
use super::super::{CodeGenCtx, RValType, ValExpectation, ValType};
use super::gen;

pub fn gen_cast(ctx: &CodeGenCtx, ty: &ASTType, val: &AST) -> ValType {
    let lhs_ty = gen(ctx, val, ValExpectation::RVal);
    let lhs_rval_ty = lhs_ty.expect_rval_ref();

    let to_type = ctx.get_ty(ty);

    match lhs_rval_ty {
        RValType::Bool => unimplemented!(),
        RValType::U8 => unimplemented!(),
        RValType::Char => unimplemented!(),
        RValType::I32 => unimplemented!(),
        RValType::F64 => unimplemented!(),
        RValType::Void => panic!("Cannot cast void type"),
        RValType::Never => panic!("Cannot cast never type"),
        RValType::Value(_) => unimplemented!(),
        RValType::GenericInst(_, _, _) => unimplemented!(),
        RValType::String => unimplemented!(),
        RValType::Class(ty) => {
            let lhs_ty = unsafe { ty.as_ref() };
            match &to_type {
                RValType::Bool
                | RValType::U8
                | RValType::Char
                | RValType::I32
                | RValType::F64
                | RValType::Void
                | RValType::Never
                | RValType::ByRef(_)
                | RValType::Array(_) => {
                    panic!("cast from {} to {} is not allowed", lhs_rval_ty, to_type)
                }
                RValType::Value(_) => unimplemented!(),
                RValType::GenericInst(_, _, _) => unimplemented!(),
                RValType::String => {
                    unimplemented!()
                }
                RValType::Class(ty) => {
                    let rhs_ty = unsafe { ty.as_ref() };

                    if lhs_ty as *const Type != rhs_ty {
                        let mut base = lhs_ty.extends;
                        let mut castable = false;
                        while let Some(base_ref) = unsafe { base.as_ref() } {
                            if base == rhs_ty {
                                castable = true;
                                break;
                            }
                            base = base_ref;
                        }

                        if !castable {
                            panic!("cast from {} to {} is not allowed", lhs_rval_ty, to_type);
                        }
                    }
                }
            }
        }
        RValType::ByRef(_) => unimplemented!(),
        RValType::Array(_) => unimplemented!(),
    }

    ValType::RVal(to_type)
}
