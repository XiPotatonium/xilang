mod branch_expr;
mod call;
mod cast;
mod literal;
mod loop_expr;
mod op;
mod sym;

use super::super::ast::{ASTType, AST};
use op::BinOp;
// use super::interpreter::constant_folding;
use super::{CodeGenCtx, RValType, SymType, SymUsage, ValType};

use xir::attrib::*;
use xir::inst::Inst;
use xir::tok::to_tok;
use xir::CTOR_NAME;

pub fn gen_base_ctor(ctx: &CodeGenCtx, args: &Vec<Box<AST>>) {
    let base = unsafe { ctx.class.extends.as_ref().unwrap() };

    // similar to gen_new
    let ctors = base.methods.get(CTOR_NAME).unwrap();
    let args_ty: Vec<RValType> = args.iter().map(|arg| gen(ctx, arg).expect_rval()).collect();

    let ctor = call::pick_method_from_refs(ctors, &args_ty);
    let ctor = if let Some(ctor) = ctor {
        ctor
    } else {
        panic!("Cannot find ctor");
    };

    let mut builder = ctx.module.builder.borrow_mut();
    let ctor_sig = builder.add_method_sig(true, &ctor.ps, &RValType::Void);
    let (ctor_idx, tok_tag) = builder.add_const_member(
        unsafe { base.parent.as_ref().unwrap().fullname() },
        &base.name,
        CTOR_NAME,
        ctor_sig,
    );

    ctx.method_builder.borrow_mut().add_inst(Inst::LdArg0); // load self
    ctx.method_builder
        .borrow_mut()
        .add_inst(Inst::Call(to_tok(ctor_idx, tok_tag)));
}

pub fn gen(ctx: &CodeGenCtx, ast: &AST) -> ValType {
    /*
    if ctx.mgr.cfg.optim >= 1 && ast.is_constant() {
        return gen(ctx, &Box::new(constant_folding(ast)));
    }
    */

    match ast {
        AST::Block(children) => gen_block(ctx, children),
        AST::ExprStmt(stmt) => gen_expr_stmt(ctx, stmt),
        AST::If(cond, then, els) => ValType::RVal(branch_expr::gen_if(ctx, cond, then, els)),
        AST::Let(pattern, flag, ty, init) => ValType::RVal(gen_let(ctx, pattern, flag, ty, init)),
        AST::Return(v) => ValType::Ret(if let ValType::RVal(ret) = gen(ctx, v) {
            ctx.method_builder.borrow_mut().add_inst(Inst::Ret);
            ret
        } else {
            unreachable!();
        }),
        AST::Loop(body) => ValType::RVal(loop_expr::gen_loop(ctx, body)),
        AST::Break(v) => loop_expr::gen_break(ctx, v),
        AST::Continue => loop_expr::gen_continue(ctx),
        AST::OpCast(ty, val) => cast::gen_cast(ctx, ty, val),
        AST::OpNew(ty, fields) => ValType::RVal(call::gen_new(ctx, ty, fields)),
        AST::OpNewArr(ty, dim) => ValType::RVal(call::gen_new_arr(ctx, ty, dim)),
        AST::OpCall(f, args) => ValType::RVal(call::gen_call(ctx, f, args)),
        AST::OpAssign(lhs, rhs) => ValType::RVal(gen_assign(ctx, lhs, rhs)),
        AST::OpNeg(lhs) => op::gen_neg(ctx, lhs),
        AST::OpLogNot(lhs) => op::gen_log_not(ctx, lhs),
        AST::OpLogAnd(lhs, rhs) => ValType::RVal(op::gen_and(ctx, lhs, rhs)),
        AST::OpLogOr(lhs, rhs) => ValType::RVal(op::gen_or(ctx, lhs, rhs)),
        AST::OpAdd(lhs, rhs) => ValType::RVal(op::gen_numeric(ctx, BinOp::Add, lhs, rhs)),
        AST::OpSub(lhs, rhs) => ValType::RVal(op::gen_numeric(ctx, BinOp::Sub, lhs, rhs)),
        AST::OpMul(lhs, rhs) => ValType::RVal(op::gen_numeric(ctx, BinOp::Mul, lhs, rhs)),
        AST::OpDiv(lhs, rhs) => ValType::RVal(op::gen_numeric(ctx, BinOp::Div, lhs, rhs)),
        AST::OpMod(lhs, rhs) => ValType::RVal(op::gen_numeric(ctx, BinOp::Mod, lhs, rhs)),
        AST::OpNe(lhs, rhs) => ValType::RVal(op::gen_cmp(ctx, BinOp::Ne, lhs, rhs)),
        AST::OpEq(lhs, rhs) => ValType::RVal(op::gen_cmp(ctx, BinOp::Eq, lhs, rhs)),
        AST::OpGe(lhs, rhs) => ValType::RVal(op::gen_cmp(ctx, BinOp::Ge, lhs, rhs)),
        AST::OpGt(lhs, rhs) => ValType::RVal(op::gen_cmp(ctx, BinOp::Gt, lhs, rhs)),
        AST::OpLe(lhs, rhs) => ValType::RVal(op::gen_cmp(ctx, BinOp::Le, lhs, rhs)),
        AST::OpLt(lhs, rhs) => ValType::RVal(op::gen_cmp(ctx, BinOp::Lt, lhs, rhs)),
        AST::OpObjAccess(_, _) => {
            let lval = sym::gen_sym(ctx, ast, SymUsage::Assignee);
            match &lval {
                SymType::Field(f) => {
                    let (mod_name, class_name, field_name, field_ty) = unsafe {
                        let f_ref = f.as_ref();
                        let class_ref = f_ref.parent.as_ref().unwrap();
                        let module_ref = class_ref.parent.as_ref().unwrap();
                        (
                            module_ref.fullname(),
                            &class_ref.name,
                            &f_ref.name,
                            f_ref.ty.clone(),
                        )
                    };
                    let sig = ctx.module.builder.borrow_mut().add_field_sig(&field_ty);
                    let (field_idx, tok_tag) = ctx
                        .module
                        .builder
                        .borrow_mut()
                        .add_const_member(mod_name, class_name, field_name, sig);
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst(Inst::LdFld(to_tok(field_idx, tok_tag)));

                    ValType::RVal(field_ty)
                }
                SymType::ArrLen => {
                    ctx.method_builder.borrow_mut().add_inst(Inst::LdLen);
                    ValType::RVal(RValType::I32)
                }
                _ => unreachable!(),
            }
        }
        AST::OpStaticAccess(_, _) => {
            let v = sym::gen_sym(ctx, ast, SymUsage::Assignee);
            gen_static_access(ctx, v)
        }
        AST::OpArrayAccess(_, _) => {
            let lval = sym::gen_sym(ctx, ast, SymUsage::Assignee);
            match lval {
                SymType::ArrAcc(ele_ty) => {
                    ctx.method_builder.borrow_mut().add_ldelem(&ele_ty);
                    ValType::RVal(ele_ty)
                }
                _ => panic!("Cannot array access {}", lval),
            }
        }
        AST::Id(id) => ValType::RVal(gen_id_rval(ctx, id)),
        AST::Bool(val) => literal::gen_bool(ctx, *val),
        AST::Int(val) => literal::gen_int(ctx, *val),
        AST::String(val) => literal::gen_string(ctx, val),
        AST::None => literal::gen_none(),
        _ => unimplemented!("{}", ast),
    }
}

/// Access a static field
fn gen_static_access(ctx: &CodeGenCtx, v: SymType) -> ValType {
    match &v {
        SymType::Field(f) => {
            let (mod_name, class_name, field_name, field_ty) = unsafe {
                let f_ref = f.as_ref();
                let class_ref = f_ref.parent.as_ref().unwrap();
                let module_ref = class_ref.parent.as_ref().unwrap();
                (
                    module_ref.fullname(),
                    &class_ref.name,
                    &f_ref.name,
                    f_ref.ty.clone(),
                )
            };
            let sig = ctx.module.builder.borrow_mut().add_field_sig(&field_ty);
            let (field_idx, tok_tag) = ctx
                .module
                .builder
                .borrow_mut()
                .add_const_member(mod_name, class_name, field_name, sig);
            ctx.method_builder
                .borrow_mut()
                .add_inst(Inst::LdSFld(to_tok(field_idx, tok_tag)));

            ValType::RVal(field_ty)
        }
        _ => unreachable!(),
    }
}

fn gen_block(ctx: &CodeGenCtx, children: &Vec<Box<AST>>) -> ValType {
    // Push Symbol table
    ctx.locals.borrow_mut().push();

    let mut ret = ValType::RVal(RValType::Void);
    for stmt in children.iter() {
        ret = gen(ctx, stmt);
        if ctx.method_builder.borrow().cur_bb_last_is_branch() {
            // do not generate unreachable stmts.
            // branch should be the last inst in bb
            break;
        }
    }

    // Pop Symbol table
    ctx.locals.borrow_mut().pop();
    ret
}

fn gen_expr_stmt(ctx: &CodeGenCtx, stmt: &Box<AST>) -> ValType {
    let ret = gen(ctx, stmt);
    match &ret {
        ValType::RVal(ty) => {
            // pop from stack
            match ty {
                RValType::Void => (),
                _ => {
                    ctx.method_builder.borrow_mut().add_inst(Inst::Pop);
                }
            };
            ValType::RVal(RValType::Void)
        }
        ValType::Ret(_) => ret,
        _ => unreachable!(),
    }
}

fn gen_let(
    ctx: &CodeGenCtx,
    pattern: &Box<AST>,
    flag: &LocalAttrib,
    ty: &ASTType,
    init: &AST,
) -> RValType {
    match pattern.as_ref() {
        AST::Id(id) => {
            if let AST::None = init {
                // no initialization
                if let ASTType::None = ty {
                    // invalid let stmt
                    panic!("Specify type or use initialization");
                } else {
                    // this variable is declared but not initialized
                    let ty = ctx.get_ty(ty);
                    ctx.locals.borrow_mut().add(id, ty.clone(), *flag, false);
                }
            } else {
                // build init
                let init_ty = gen(ctx, init).expect_rval();
                let offset = ctx
                    .locals
                    .borrow_mut()
                    .add(id, init_ty.clone(), *flag, true);
                ctx.method_builder.borrow_mut().add_inst_stloc(offset);

                if let ASTType::None = ty {
                    // no type, induce type from return value of init
                } else {
                    // check type match
                    let ty = ctx.get_ty(ty);
                    if ty != init_ty {
                        panic!("Cannot assign {} to local var {}: {}", init_ty, id, ty);
                    }
                }
            }
        }
        AST::TuplePattern(_) => {
            unimplemented!()
        }
        _ => unreachable!(),
    };

    RValType::Void
}

fn gen_assign(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let lval = sym::gen_sym(ctx, lhs, SymUsage::Assignee);
    let v_ty = gen(ctx, rhs).expect_rval();

    match lval {
        SymType::Local(idx) => {
            let locals = ctx.locals.borrow();
            let local = &locals.locals[idx];
            let local_ty = local.ty.clone();

            if local_ty != v_ty {
                panic!("Cannot assign {} to local {}: {}", v_ty, local.id, local_ty);
            }

            ctx.method_builder.borrow_mut().add_inst_stloc(local.idx);
        }
        SymType::KwLSelf => {
            // lval guarentee that we are in instance method
            // ctx.method_builder.borrow_mut().add_inst_starg(0);
            panic!("Cannot assign self");
        }
        SymType::Arg(idx) => {
            let arg = &ctx.method.ps[idx];

            if arg.ty != v_ty {
                panic!("Cannot assign {} to arg {}: {}", v_ty, arg.id, arg.ty);
            }

            ctx.method_builder.borrow_mut().add_inst_starg(if ctx
                .method
                .attrib
                .is(MethodAttribFlag::Static)
            {
                idx
            } else {
                idx + 1
            } as u16);
        }
        SymType::Field(f) => {
            // TODO private and public
            let (mod_name, class_name, f_ref) = unsafe {
                let f_ref = f.as_ref();
                let class_ref = f_ref.parent.as_ref().unwrap();
                let module_ref = class_ref.parent.as_ref().unwrap();
                (module_ref.fullname(), &class_ref.name, f_ref)
            };

            if f_ref.ty != v_ty {
                panic!("Cannot assign {} value to {}", v_ty, f_ref);
            }

            let sig = ctx.module.builder.borrow_mut().add_field_sig(&f_ref.ty);
            let (f_idx, tok_tag) = ctx.module.builder.borrow_mut().add_const_member(
                mod_name,
                class_name,
                &f_ref.name,
                sig,
            );
            let inst = if f_ref.attrib.is(FieldAttribFlag::Static) {
                Inst::StSFld(to_tok(f_idx, tok_tag))
            } else {
                Inst::StFld(to_tok(f_idx, tok_tag))
            };

            ctx.method_builder.borrow_mut().add_inst(inst);
        }
        SymType::ArrAcc(ele_ty) => {
            if ele_ty != v_ty {
                panic!("Cannot store {} into {} array", v_ty, ele_ty);
            }
            ctx.method_builder.borrow_mut().add_stelem(&ele_ty);
        }
        SymType::Module(_) => panic!(),
        SymType::Class(_) => panic!(),
        _ => unreachable!(),
    }

    // assign op has no value left on evaluation stack
    RValType::Void
}

fn gen_id_rval(ctx: &CodeGenCtx, id: &str) -> RValType {
    // try search locals
    {
        let locals = ctx.locals.borrow();
        let is_instance_method = !ctx.method.attrib.is(MethodAttribFlag::Static);
        if id == "self" {
            if is_instance_method {
                // first argument
                ctx.method_builder.borrow_mut().add_inst_ldarg(0);
                return RValType::Obj(
                    ctx.module.get_module().fullname().to_owned(),
                    ctx.class.name.clone(),
                );
            } else {
                panic!("Invalid self keyword in static method");
            }
        } else if let Some(local_var) = locals.get(id) {
            ctx.method_builder
                .borrow_mut()
                .add_inst_ldloc(local_var.idx);
            return local_var.ty.clone();
        } else if let Some(arg_idx) = ctx.ps_map.get(id) {
            let arg = &ctx.method.ps[*arg_idx];
            ctx.method_builder
                .borrow_mut()
                .add_inst_ldarg(if is_instance_method {
                    *arg_idx + 1
                } else {
                    *arg_idx
                } as u16);
            return arg.ty.clone();
        }
    }
    unimplemented!("{} is not local nor arg", id);
}
