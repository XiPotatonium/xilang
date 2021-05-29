use super::super::super::ast::AST;
use super::super::{gen, CodeGenCtx, RValType, ValExpectation, ValType};

use xir::Inst;

pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Ge,
    Gt,
    Le,
    Lt,
}

pub fn gen_neg(ctx: &CodeGenCtx, lhs: &Box<AST>) -> ValType {
    let v_ty = gen(ctx, lhs, ValExpectation::RVal);

    match v_ty.expect_rval_ref() {
        RValType::I32 | RValType::F64 => {
            ctx.method_builder.borrow_mut().add_inst(Inst::Neg);
        }
        _ => panic!("neg op is only available for i32 or f64 operand"),
    };

    v_ty
}

pub fn gen_log_not(ctx: &CodeGenCtx, lhs: &Box<AST>) -> ValType {
    let v_ty = gen(ctx, lhs, ValExpectation::RVal);

    match v_ty.expect_rval_ref() {
        RValType::Bool => {
            ctx.method_builder
                .borrow_mut()
                .add_inst_ldc(0)
                .add_inst(Inst::CEq);
        }
        _ => panic!("not op is only available for bool operand"),
    };

    v_ty
}

pub fn gen_and(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let rhs_bb;
    let false_bb;
    let after_bb;
    {
        let mut builder = ctx.method_builder.borrow_mut();
        after_bb = builder.insert_after_cur();
        false_bb = builder.insert_after_cur();
        rhs_bb = builder.insert_after_cur();
    }

    let lhs_ty = gen(ctx, lhs, ValExpectation::RVal);
    match lhs_ty.expect_rval() {
        RValType::Bool => {}
        _ => panic!("Cond not return bool"),
    }

    ctx.method_builder
        .borrow_mut()
        .add_brfalse(false_bb.clone())
        .set_cur_bb(rhs_bb);

    let rhs_ty = gen(ctx, rhs, ValExpectation::RVal);
    match rhs_ty.expect_rval() {
        RValType::Bool => {}
        _ => panic!("Cond not return bool"),
    }

    let mut builder = ctx.method_builder.borrow_mut();
    builder.add_br(after_bb.clone()).set_cur_bb(false_bb);

    builder
        .add_inst_ldc(0)
        .add_br(after_bb.clone())
        .set_cur_bb(after_bb);

    RValType::Bool
}

pub fn gen_or(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let rhs_bb;
    let false_bb;
    let after_bb;
    {
        let mut builder = ctx.method_builder.borrow_mut();
        after_bb = builder.insert_after_cur();
        false_bb = builder.insert_after_cur();
        rhs_bb = builder.insert_after_cur();
    }

    let lhs_ty = gen(ctx, lhs, ValExpectation::RVal);
    match lhs_ty.expect_rval() {
        RValType::Bool => {}
        _ => panic!("Cond not return bool"),
    }

    ctx.method_builder
        .borrow_mut()
        .add_brtrue(after_bb.clone())
        .set_cur_bb(rhs_bb);

    let rhs_ty = gen(ctx, rhs, ValExpectation::RVal);
    match rhs_ty.expect_rval() {
        RValType::Bool => {}
        _ => panic!("Cond not return bool"),
    }

    let mut builder = ctx.method_builder.borrow_mut();
    builder.add_br(after_bb.clone()).set_cur_bb(false_bb);

    builder
        .add_inst_ldc(0)
        .add_br(after_bb.clone())
        .set_cur_bb(after_bb);

    RValType::Bool
}

pub fn gen_numeric(ctx: &CodeGenCtx, op: BinOp, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let lty = gen(ctx, lhs, ValExpectation::RVal).expect_rval();
    let rty = gen(ctx, rhs, ValExpectation::RVal).expect_rval();

    if lty != rty {
        panic!("Numeric op cannot be applied between {} and {}", lty, rty);
    }

    // TODO: check lty

    ctx.method_builder.borrow_mut().add_inst(match op {
        BinOp::Add => Inst::Add,
        BinOp::Sub => Inst::Sub,
        BinOp::Mul => Inst::Mul,
        BinOp::Div => Inst::Div,
        BinOp::Mod => Inst::Rem,
        _ => unreachable!(),
    });
    lty
}

pub fn gen_cmp(ctx: &CodeGenCtx, op: BinOp, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let lty = gen(ctx, lhs, ValExpectation::RVal).expect_rval();
    let rty = gen(ctx, rhs, ValExpectation::RVal).expect_rval();

    if lty != rty {
        panic!("Cmp op cannot be applied between {} and {}", lty, rty);
    }

    // TODO: check lty

    let mut builder = ctx.method_builder.borrow_mut();

    match op {
        BinOp::Eq => {
            builder.add_inst(Inst::CEq);
        }
        BinOp::Ne => {
            builder
                .add_inst(Inst::CEq)
                .add_inst(Inst::LdC0)
                .add_inst(Inst::CEq);
        }
        BinOp::Gt => {
            builder.add_inst(Inst::CGt);
        }
        BinOp::Le => {
            builder
                .add_inst(Inst::CGt)
                .add_inst(Inst::LdC0)
                .add_inst(Inst::CEq);
        }
        BinOp::Lt => {
            builder.add_inst(Inst::CLt);
        }
        BinOp::Ge => {
            builder
                .add_inst(Inst::CLt)
                .add_inst(Inst::LdC0)
                .add_inst(Inst::CEq);
        }
        _ => unreachable!(),
    }

    RValType::Bool
}
