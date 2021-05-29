use super::super::super::ast::AST;
use super::super::{CodeGenCtx, LoopCtx, LoopType, RValType, ValExpectation, ValType};
use super::gen;

pub fn gen_continue(ctx: &CodeGenCtx) -> ValType {
    if let Some(l) = ctx.loop_ctx.borrow_mut().last_mut() {
        ctx.method_builder
            .borrow_mut()
            .add_br(l.break_target.clone());
    } else {
        panic!("Break not in a loop expr");
    }
    ValType::RVal(RValType::Void)
}

pub fn gen_break(ctx: &CodeGenCtx, v: &AST) -> ValType {
    if let AST::None = v {
        if let Some(l) = ctx.loop_ctx.borrow_mut().last_mut() {
            if let LoopType::Loop(ty) = &mut l.ty {
                match ty {
                            RValType::Void => {}
                            RValType::Never => {
                                l.ty = LoopType::Loop(RValType::Void);
                            },
                            _ => panic!("Loop return type mismatch. Previously break with {} but later break with {}", ty, RValType::Void),
                        };
            } else {
                unimplemented!();
            }
            ctx.method_builder
                .borrow_mut()
                .add_br(l.break_target.clone());
        } else {
            panic!("Break not in a loop expr");
        }

        ValType::RVal(RValType::Void)
    } else {
        let v_ty = gen(ctx, v, ValExpectation::RVal);

        if let ValType::RVal(v_ty_) = &v_ty {
            if let Some(l) = ctx.loop_ctx.borrow_mut().last_mut() {
                if let LoopType::Loop(ty) = &mut l.ty {
                    match ty {
                        RValType::Never => {
                            l.ty = LoopType::Loop(v_ty_.clone());
                        }
                        _ => {
                            if v_ty_ != ty {
                                panic!("Loop return type mismatch. Previously break with {} but later break with {}", ty, v_ty_);
                            }
                        }
                    };
                } else {
                    panic!("break with expr is only allowed in loop");
                }
                ctx.method_builder
                    .borrow_mut()
                    .add_br(l.break_target.clone());
            } else {
                panic!("Break not in a loop expr");
            }
        } else {
            panic!();
        }

        v_ty
    }
}

pub fn gen_loop(ctx: &CodeGenCtx, body: &AST, expectation: ValExpectation) -> RValType {
    {
        let mut builder = ctx.method_builder.borrow_mut();
        let after_bb = builder.insert_after_cur();
        let body_bb = builder.insert_after_cur();
        builder.set_cur_bb(body_bb.clone());
        ctx.loop_ctx.borrow_mut().push(LoopCtx {
            ty: LoopType::Loop(RValType::Never),
            continue_target: body_bb,
            break_target: after_bb,
        });
    }

    gen(ctx, body, expectation);

    {
        let LoopCtx {
            ty,
            continue_target: body_bb,
            break_target: after_bb,
        } = ctx.loop_ctx.borrow_mut().pop().unwrap();
        let mut builder = ctx.method_builder.borrow_mut();

        if !builder.cur_bb_last_is_branch() {
            // loop if no branch
            builder.add_br(body_bb);
        }

        builder.set_cur_bb(after_bb);

        if let LoopType::Loop(ret) = ty {
            ret
        } else {
            RValType::Void
        }
    }
}
