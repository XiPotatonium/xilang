use super::super::class::CodeGenCtx;
use super::ast::{Op, AST};
use super::interpreter::const_collapse;
use crate::ir::flag::*;
use crate::ir::var::VarType;

pub fn gen(ctx: &CodeGenCtx, ast: &Box<AST>) -> VarType {
    if ast.is_const() {
        gen(ctx, &const_collapse(ast))
    } else {
        match ast.as_ref() {
            AST::Block(stmts) => gen_block(ctx, stmts),
            AST::Let(pattern, ty, flag, init) => gen_let(ctx, pattern, ty, flag, init),
            AST::Unary(op, o1) => gen_unary(ctx, op, o1),
            AST::Binary(op, o1, o2) => gen_binary(ctx, op, o1, o2),
            AST::Call(f, args) => gen_call(ctx, f, args),
            AST::Cast(ty, v) => gen_cast(ctx, ty, v),
            AST::None => VarType::Void,
            _ => unimplemented!(),
        }
    }
}

fn gen_block(ctx: &CodeGenCtx, stmts: &Vec<Box<AST>>) -> VarType {
    // Push Symbol table

    let mut ret = VarType::Void;
    for stmt in stmts.iter() {
        ret = gen(ctx, stmt);
    }

    // Pop Symbol table
    ret
}

fn gen_let(
    ctx: &CodeGenCtx,
    pattern: &Box<AST>,
    ty: &Box<AST>,
    flag: &Flag,
    init: &Box<AST>,
) -> VarType {
    match pattern.as_ref() {
        AST::Id(id) => {
            if let AST::None = init.as_ref() {
                // no initialization
                if let AST::None = ty.as_ref() {
                    // invalid let stmt
                    panic!("Specify type or use initialization");
                } else {
                    // this variable is declared but not initialized
                    let ty = ctx.class.get_type(ty, ctx.mgr);
                    ty
                }
            } else {
                // build init
                let init_ty = gen(ctx, init);

                if let AST::None = ty.as_ref() {
                    // no type, induce type from return value of init
                } else {
                    // check type match
                    let ty = ctx.class.get_type(ty, ctx.mgr);
                }

                init_ty
            }
        }
        AST::TuplePattern(_) => {
            unimplemented!()
        }
        _ => unreachable!(),
    }
}

fn gen_unary(ctx: &CodeGenCtx, op: &Op, o1: &Box<AST>) -> VarType {
    unimplemented!();
}

fn gen_binary(ctx: &CodeGenCtx, op: &Op, o1: &Box<AST>, o2: &Box<AST>) -> VarType {
    unimplemented!();
}

fn gen_call(ctx: &CodeGenCtx, f: &Box<AST>, args: &Vec<Box<AST>>) -> VarType {
    unimplemented!();
}

fn gen_cast(ctx: &CodeGenCtx, ty: &Box<AST>, v: &Box<AST>) -> VarType {
    unimplemented!();
}
