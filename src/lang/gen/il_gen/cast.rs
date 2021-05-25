use super::super::super::ast::{ASTType, AST};
use super::super::super::mod_mgr::Class;
use super::super::{CodeGenCtx, RValType, ValType};
use super::gen;

pub fn gen_cast(ctx: &CodeGenCtx, ty: &ASTType, val: &AST) -> ValType {
    let lhs_ty = gen(ctx, val);
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
        RValType::String => unimplemented!(),
        RValType::Obj(mod_fullname, class_name) => {
            let lhs_class = ctx
                .mgr
                .mod_tbl
                .get(mod_fullname)
                .unwrap()
                .classes
                .get(class_name)
                .unwrap()
                .as_ref();
            match &to_type {
                RValType::Bool
                | RValType::U8
                | RValType::Char
                | RValType::I32
                | RValType::F64
                | RValType::Void
                | RValType::Never
                | RValType::String
                | RValType::Array(_) => {
                    panic!("cast from {} to {} is not allowed", lhs_rval_ty, to_type)
                }
                RValType::Obj(mod_fullname, class_name) => {
                    let rhs_class = ctx
                        .mgr
                        .mod_tbl
                        .get(mod_fullname)
                        .unwrap()
                        .classes
                        .get(class_name)
                        .unwrap()
                        .as_ref();

                    if lhs_class as *const Class != rhs_class {
                        let mut base = lhs_class.extends;
                        let mut castable = false;
                        while let Some(base_ref) = unsafe { base.as_ref() } {
                            if base == rhs_class {
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
        RValType::Array(_) => unimplemented!(),
    }

    ValType::RVal(to_type)
}
