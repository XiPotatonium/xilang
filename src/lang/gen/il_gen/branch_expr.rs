use super::super::super::ast::AST;
use super::super::{CodeGenCtx, RValType, ValExpectation, ValType};
use super::gen;

pub fn gen_if(
    ctx: &CodeGenCtx,
    cond: &AST,
    then: &AST,
    els: &AST,
    expectation: ValExpectation,
) -> RValType {
    let then_bb;
    let els_bb;
    let after_bb;
    {
        let mut builder = ctx.method_builder.borrow_mut();
        after_bb = builder.insert_after_cur();
        els_bb = builder.insert_after_cur();
        then_bb = builder.insert_after_cur();
    }

    let cond_ty = gen(ctx, cond, ValExpectation::RVal);
    match cond_ty.expect_rval() {
        RValType::Bool => {}
        _ => panic!("Cond not return bool"),
    }

    {
        let mut builder = ctx.method_builder.borrow_mut();
        builder.add_brfalse(els_bb.clone());
        builder.set_cur_bb(then_bb);
    }

    let then_v = gen(ctx, then, expectation);

    {
        let mut builder = ctx.method_builder.borrow_mut();
        if !builder.cur_bb_last_is_branch() {
            // branch to after if no branch
            builder.add_br(after_bb.clone());
        }
        builder.set_cur_bb(els_bb);
    }

    let els_v = gen(ctx, els, expectation);

    {
        let mut builder = ctx.method_builder.borrow_mut();
        builder.set_cur_bb(after_bb);
    }

    let ret = if let ValType::RVal(then_v) = then_v {
        if let ValType::RVal(els_v) = els_v {
            if then_v != els_v {
                panic!("Mismatch then type ({}) and else type ({})", then_v, els_v);
            } else {
                then_v
            }
        } else {
            panic!("Mismatch then type ({}) and else type ({})", then_v, els_v);
        }
    } else {
        RValType::Void
    };

    ret
}
