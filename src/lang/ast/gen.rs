use super::super::class::CodeGenCtx;
use super::ast::AST;
use crate::ir::flag::*;
use crate::ir::ty::VarType;

pub fn gen(ctx: &CodeGenCtx, ast: &Box<AST>) -> VarType {
    /*
    // TODO Use const collapse
    if ast.is_const() {
        gen(ctx, &Box::new(const_collapse(ast)))
    } else {

    }
    */
    match ast.as_ref() {
        AST::Block(stmts) => gen_block(ctx, stmts),
        AST::Let(pattern, flag, ty, init) => gen_let(ctx, pattern, flag, ty, init),
        AST::OpNew(ty, fields) => gen_new(ctx, ty, fields),
        AST::OpCall(f, args) => gen_call(ctx, f, args),
        AST::OpCast(ty, v) => gen_cast(ctx, ty, v),
        AST::None => VarType::Void,
        _ => {
            println!("{}", ast);
            unimplemented!()
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
    flag: &Flag,
    ty: &Box<AST>,
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
                    ctx.method
                        .locals
                        .borrow_mut()
                        .add(id, ty.clone(), *flag, false);
                    ty
                }
            } else {
                // build init
                let init_ty = gen(ctx, init);
                let offset = ctx
                    .method
                    .locals
                    .borrow_mut()
                    .add(id, init_ty.clone(), *flag, true);
                ctx.class.builder.borrow_mut().add_inst_store(
                    ctx.method.method_idx,
                    &init_ty,
                    offset,
                );

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

fn gen_call(ctx: &CodeGenCtx, f: &Box<AST>, args: &Vec<Box<AST>>) -> VarType {
    unimplemented!();
}

fn gen_new(ctx: &CodeGenCtx, ty: &Box<AST>, fields: &Vec<Box<AST>>) -> VarType {
    let ret = ctx.class.get_type(ty, ctx.mgr);
    match &ret {
        VarType::Class(class_name) => {
            // Call <init> of that class
            let builder = ctx.class.builder.borrow_mut();
            unimplemented!();
        },
        VarType::Array(inner_ty) => unimplemented!(),
        _ => unreachable!("Invalid new expression, only new class or array is allowed"),
    }
    ret
}

fn gen_cast(ctx: &CodeGenCtx, ty: &Box<AST>, v: &Box<AST>) -> VarType {
    unimplemented!();
}
