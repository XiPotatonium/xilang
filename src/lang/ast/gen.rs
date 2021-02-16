use super::super::class::CodeGenCtx;
use super::ast::AST;
use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::ty::{VarType, XirType};

pub fn gen(ctx: &CodeGenCtx, ast: &Box<AST>) -> VarType {
    /*
    // TODO Use constant folding
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
        AST::OpAssign(lhs, rhs) => gen_assign(ctx, lhs, rhs),
        AST::OpAdd(lhs, rhs) => gen_add(ctx, lhs, rhs),
        AST::Id(id) => gen_id_rval(ctx, id),
        AST::Int(val) => {
            ctx.class
                .builder
                .borrow_mut()
                .add_inst_pushi(ctx.method.method_idx, *val);
            VarType::Int
        }
        AST::None => VarType::Void,
        _ => unimplemented!(),
    }
}

fn gen_lval(ctx: &CodeGenCtx, ast: &Box<AST>) -> Vec<XirType> {
    match ast.as_ref() {
        AST::Id(id) => gen_id_lval(ctx, id),
        AST::OpObjAccess(lhs, rhs) => {
            unimplemented!();
        }
        AST::OpStaticAccess(lhs, rhs) => {
            unimplemented!();
        }
        AST::OpArrayAccess(lhs, rhs) => {
            unimplemented!();
        }
        _ => unimplemented!(),
    }
}

fn gen_block(ctx: &CodeGenCtx, stmts: &Vec<Box<AST>>) -> VarType {
    // Push Symbol table
    ctx.locals.borrow_mut().push();

    let mut ret = VarType::Void;
    for stmt in stmts.iter() {
        ret = gen(ctx, stmt);
    }

    // Pop Symbol table
    ctx.locals.borrow_mut().pop();
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
                    ctx.locals.borrow_mut().add(id, ty.clone(), *flag, false);
                }
            } else {
                // build init
                let init_ty = gen(ctx, init);
                let offset = ctx
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
                    if ty != init_ty {
                        panic!("Invalid let statement: Incompatible type");
                    }
                }
            }
        }
        AST::TuplePattern(_) => {
            unimplemented!()
        }
        _ => unreachable!(),
    };

    VarType::Void
}

fn gen_call(ctx: &CodeGenCtx, f: &Box<AST>, args: &Vec<Box<AST>>) -> VarType {
    let args_ty: Vec<VarType> = args.iter().map(|arg| gen(ctx, arg)).collect();
    let possible_lval = gen_lval(ctx, f);
    let ret: Option<VarType> = None;

    for lval in possible_lval.iter() {
        match &lval {
            XirType::RVal(_) => unreachable!(),
            XirType::Method(class_name, method_name) => {
                unimplemented!();
            }
            _ => (),
        }
    }

    if let Some(ret) = ret {
        ret
    } else {
        panic!("Invalid call. Cannot find appropriate method");
    }
}

fn gen_new(ctx: &CodeGenCtx, ty: &Box<AST>, fields: &Vec<Box<AST>>) -> VarType {
    let ret = ctx.class.get_type(ty, ctx.mgr);
    match &ret {
        VarType::Class(class_name) => {
            let builder = ctx.class.builder.borrow_mut();
            unimplemented!();
        }
        VarType::Array(inner_ty) => unimplemented!(),
        _ => panic!("Invalid new expression, only new class or array is allowed"),
    }
    ret
}

fn gen_assign(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> VarType {
    let rval_ty = gen(ctx, rhs);
    let lval_ty = gen_lval(ctx, lhs);
    unimplemented!();
}

fn gen_add(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> VarType {
    let lty = gen(ctx, lhs);
    let rty = gen(ctx, rhs);

    if lty != rty {
        panic!("Cannot add between {} and {}", lty, rty);
    }

    match &lty {
        VarType::Int => {
            ctx.class
                .builder
                .borrow_mut()
                .add_inst(ctx.method.method_idx, Inst::IAdd);
            lty
        }
        _ => unimplemented!(),
    }
}

fn gen_id_rval(ctx: &CodeGenCtx, id: &String) -> VarType {
    // try search locals
    {
        let locals = ctx.locals.borrow();
        if let Some(local_idx) = locals.sym_tbl.last().unwrap().get(id) {
            let local = &locals.locals[*local_idx];
            ctx.class.builder.borrow_mut().add_inst_load(
                ctx.method.method_idx,
                &local.ty,
                local.offset,
            );
            return local.ty.clone();
        }
    }
    unimplemented!();
}

fn gen_id_lval(ctx: &CodeGenCtx, id: &String) -> Vec<XirType> {
    unimplemented!();
}
