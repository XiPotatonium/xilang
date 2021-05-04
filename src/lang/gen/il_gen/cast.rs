use super::super::super::ast::AST;
use super::super::{CodeGenCtx, RValType, ValType};
use super::gen;

pub fn gen_cast(ctx: &CodeGenCtx, ty: &AST, val: &AST) -> ValType {
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
        RValType::Obj(mod_fullname, class_name) => {
            let lhs_class = ctx
                .mgr
                .mod_tbl
                .get(mod_fullname)
                .unwrap()
                .get_class(class_name)
                .unwrap();
            match &to_type {
                RValType::Bool
                | RValType::U8
                | RValType::Char
                | RValType::I32
                | RValType::F64
                | RValType::Void
                | RValType::Never
                | RValType::Array(_) => {
                    panic!("cast from {} to {} is not allowed", lhs_rval_ty, to_type)
                }
                RValType::Obj(mod_fullname, class_name) => {
                    let rhs_class = ctx
                        .mgr
                        .mod_tbl
                        .get(mod_fullname)
                        .unwrap()
                        .get_class(class_name)
                        .unwrap();
                    let rhs_class = unsafe { rhs_class.as_ref().unwrap() };

                    if lhs_class != rhs_class {
                        let mut base = unsafe { lhs_class.as_ref().unwrap().extends };
                        let mut castable = false;
                        while let Some(base_ptr) = base {
                            if base_ptr == rhs_class {
                                castable = true;
                                break;
                            }
                            base = unsafe { base_ptr.as_ref().unwrap().extends };
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
