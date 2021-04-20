use super::super::ast::AST;
use super::super::mod_mgr::{FieldRef, MethodRef, Param};
use super::interpreter::constant_folding;
use super::lval::{gen_lval, gen_path_lval};
use super::op::BinOp;
use super::{CodeGenCtx, LoopCtx, LoopType, RValType, ValType};

use xir::attrib::*;
use xir::inst::Inst;
use xir::tok::to_tok;
use xir::util::path::IModPath;
use xir::CTOR_NAME;

pub fn gen(ctx: &CodeGenCtx, ast: &Box<AST>) -> ValType {
    if ctx.cfg.optim >= 1 && ast.is_constant() {
        return gen(ctx, &Box::new(constant_folding(ast)));
    }

    match ast.as_ref() {
        AST::Block(children) => gen_block(ctx, children),
        AST::ExprStmt(stmt) => gen_expr_stmt(ctx, stmt),
        AST::If(cond, then, els) => ValType::RVal(gen_if(ctx, cond, then, els)),
        AST::Let(pattern, flag, ty, init) => ValType::RVal(gen_let(ctx, pattern, flag, ty, init)),
        AST::Loop(body) => ValType::RVal(gen_loop(ctx, body)),
        AST::Return(v) => ValType::Ret(if let ValType::RVal(ret) = gen(ctx, v) {
            ctx.method_builder.borrow_mut().add_inst(Inst::Ret);
            ret
        } else {
            unreachable!();
        }),
        AST::Break(v) => {
            if let AST::None = v.as_ref() {
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
                let v_ty = gen(ctx, v);

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
        AST::Continue => {
            if let Some(l) = ctx.loop_ctx.borrow_mut().last_mut() {
                ctx.method_builder
                    .borrow_mut()
                    .add_br(l.break_target.clone());
            } else {
                panic!("Break not in a loop expr");
            }
            ValType::RVal(RValType::Void)
        }
        AST::OpNew(ty, fields) => ValType::RVal(gen_new(ctx, ty, fields)),
        AST::OpCall(f, args) => ValType::RVal(gen_call(ctx, f, args)),
        AST::OpAssign(lhs, rhs) => ValType::RVal(gen_assign(ctx, lhs, rhs)),
        AST::OpNeg(lhs) => {
            let v_ty = gen(ctx, lhs);

            match v_ty.expect_rval_ref() {
                RValType::I32 | RValType::F64 => {
                    ctx.method_builder.borrow_mut().add_inst(Inst::Neg);
                }
                _ => panic!("neg op is only available for i32 or f64 operand"),
            };

            v_ty
        }
        AST::OpLogNot(lhs) => {
            let v_ty = gen(ctx, lhs);

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
        AST::OpLogAnd(lhs, rhs) => ValType::RVal(gen_and(ctx, lhs, rhs)),
        AST::OpLogOr(lhs, rhs) => ValType::RVal(gen_or(ctx, lhs, rhs)),
        AST::OpAdd(lhs, rhs) => ValType::RVal(gen_numeric(ctx, BinOp::Add, lhs, rhs)),
        AST::OpSub(lhs, rhs) => ValType::RVal(gen_numeric(ctx, BinOp::Sub, lhs, rhs)),
        AST::OpMul(lhs, rhs) => ValType::RVal(gen_numeric(ctx, BinOp::Mul, lhs, rhs)),
        AST::OpDiv(lhs, rhs) => ValType::RVal(gen_numeric(ctx, BinOp::Div, lhs, rhs)),
        AST::OpMod(lhs, rhs) => ValType::RVal(gen_numeric(ctx, BinOp::Mod, lhs, rhs)),
        AST::OpNe(lhs, rhs) => ValType::RVal(gen_cmp(ctx, BinOp::Ne, lhs, rhs)),
        AST::OpEq(lhs, rhs) => ValType::RVal(gen_cmp(ctx, BinOp::Eq, lhs, rhs)),
        AST::OpGe(lhs, rhs) => ValType::RVal(gen_cmp(ctx, BinOp::Ge, lhs, rhs)),
        AST::OpGt(lhs, rhs) => ValType::RVal(gen_cmp(ctx, BinOp::Gt, lhs, rhs)),
        AST::OpLe(lhs, rhs) => ValType::RVal(gen_cmp(ctx, BinOp::Le, lhs, rhs)),
        AST::OpLt(lhs, rhs) => ValType::RVal(gen_cmp(ctx, BinOp::Lt, lhs, rhs)),
        AST::OpObjAccess(_, _) => {
            let lval = gen_lval(ctx, ast, false);
            match &lval {
                ValType::Field(mod_name, class_name, field_name) => {
                    let ret = match ctx
                        .mgr
                        .mod_tbl
                        .get(mod_name)
                        .unwrap()
                        .get_class(class_name)
                        .unwrap()
                        .get_field(field_name)
                        .unwrap()
                    {
                        FieldRef::Field(f) => f.ty.clone(),
                        FieldRef::ExtField(f) => f.ty.clone(),
                    };
                    let sig = ctx.module.builder.borrow_mut().add_field_sig(&ret);
                    let (field_idx, tok_tag) = ctx
                        .module
                        .builder
                        .borrow_mut()
                        .add_const_member(mod_name, class_name, field_name, sig);
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst(Inst::LdFld(to_tok(field_idx, tok_tag)));

                    ValType::RVal(ret)
                }
                _ => unreachable!(),
            }
        }
        AST::OpStaticAccess(_, _) => {
            let v = gen_lval(ctx, ast, false);
            gen_static_access(ctx, v)
        }
        AST::Path(p) => {
            if p.len() == 1 {
                ValType::RVal(gen_id_rval(ctx, p.as_str()))
            } else {
                let v = gen_path_lval(ctx, p, false);
                gen_static_access(ctx, v)
            }
        }
        AST::Id(id) => ValType::RVal(gen_id_rval(ctx, id)),
        AST::Bool(val) => {
            ctx.method_builder
                .borrow_mut()
                .add_inst_ldc(if *val { 1 } else { 0 });
            ValType::RVal(RValType::Bool)
        }
        AST::Int(val) => {
            ctx.method_builder.borrow_mut().add_inst_ldc(*val);
            ValType::RVal(RValType::I32)
        }
        AST::None => ValType::RVal(RValType::Void),
        _ => unimplemented!("{}", ast),
    }
}

/// Access a static field
fn gen_static_access(ctx: &CodeGenCtx, v: ValType) -> ValType {
    match &v {
        ValType::Field(mod_name, class_name, field_name) => {
            let ret = match ctx
                .mgr
                .mod_tbl
                .get(mod_name)
                .unwrap()
                .get_class(class_name)
                .unwrap()
                .get_field(field_name)
                .unwrap()
            {
                FieldRef::Field(f) => f.ty.clone(),
                FieldRef::ExtField(f) => f.ty.clone(),
            };
            let sig = ctx.module.builder.borrow_mut().add_field_sig(&ret);
            let (field_idx, tok_tag) = ctx
                .module
                .builder
                .borrow_mut()
                .add_const_member(mod_name, class_name, field_name, sig);
            ctx.method_builder
                .borrow_mut()
                .add_inst(Inst::LdSFld(to_tok(field_idx, tok_tag)));

            ValType::RVal(ret)
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

fn gen_and(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let rhs_bb;
    let false_bb;
    let after_bb;
    {
        let mut builder = ctx.method_builder.borrow_mut();
        after_bb = builder.insert_after_cur();
        false_bb = builder.insert_after_cur();
        rhs_bb = builder.insert_after_cur();
    }

    let lhs_ty = gen(ctx, lhs);
    match lhs_ty.expect_rval() {
        RValType::Bool => {}
        _ => panic!("Cond not return bool"),
    }

    ctx.method_builder
        .borrow_mut()
        .add_brfalse(false_bb.clone())
        .set_cur_bb(rhs_bb);

    let rhs_ty = gen(ctx, rhs);
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

fn gen_or(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let rhs_bb;
    let false_bb;
    let after_bb;
    {
        let mut builder = ctx.method_builder.borrow_mut();
        after_bb = builder.insert_after_cur();
        false_bb = builder.insert_after_cur();
        rhs_bb = builder.insert_after_cur();
    }

    let lhs_ty = gen(ctx, lhs);
    match lhs_ty.expect_rval() {
        RValType::Bool => {}
        _ => panic!("Cond not return bool"),
    }

    ctx.method_builder
        .borrow_mut()
        .add_brtrue(after_bb.clone())
        .set_cur_bb(rhs_bb);

    let rhs_ty = gen(ctx, rhs);
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

fn gen_if(ctx: &CodeGenCtx, cond: &Box<AST>, then: &Box<AST>, els: &Box<AST>) -> RValType {
    let then_bb;
    let els_bb;
    let after_bb;
    {
        let mut builder = ctx.method_builder.borrow_mut();
        after_bb = builder.insert_after_cur();
        els_bb = builder.insert_after_cur();
        then_bb = builder.insert_after_cur();
    }

    let cond_ty = gen(ctx, cond);
    match cond_ty.expect_rval() {
        RValType::Bool => {}
        _ => panic!("Cond not return bool"),
    }

    {
        let mut builder = ctx.method_builder.borrow_mut();
        builder.add_brfalse(els_bb.clone());
        builder.set_cur_bb(then_bb);
    }

    let then_v = gen(ctx, then);

    {
        let mut builder = ctx.method_builder.borrow_mut();
        if !builder.cur_bb_last_is_branch() {
            // branch to after if no branch
            builder.add_br(after_bb.clone());
        }
        builder.set_cur_bb(els_bb);
    }

    let els_v = gen(ctx, els);

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

fn gen_loop(ctx: &CodeGenCtx, body: &Box<AST>) -> RValType {
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

    gen(ctx, body);

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

fn gen_let(
    ctx: &CodeGenCtx,
    pattern: &Box<AST>,
    flag: &LocalAttrib,
    ty: &Box<AST>,
    init: &Box<AST>,
) -> RValType {
    match pattern.as_ref() {
        AST::Id(id) => {
            if let AST::None = init.as_ref() {
                // no initialization
                if let AST::None = ty.as_ref() {
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

                if let AST::None = ty.as_ref() {
                    // no type, induce type from return value of init
                } else {
                    // check type match
                    let ty = ctx.get_ty(ty);
                    if ty != init_ty {
                        panic!(
                            "Invalid let statement: Incompatible type {} and {}",
                            ty, init_ty
                        );
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

fn build_args(ctx: &CodeGenCtx, ps: &Vec<Param>, args: &Vec<Box<AST>>) {
    let args_ty: Vec<RValType> = args.iter().map(|arg| gen(ctx, arg).expect_rval()).collect();

    for (i, (p, arg_ty)) in ps.iter().zip(args_ty.iter()).enumerate() {
        if &p.ty != arg_ty {
            panic!(
                "Call parameter type mismatch, expect {} but found {} at {}",
                p.ty, arg_ty, i
            );
        }
    }
}

fn gen_call(ctx: &CodeGenCtx, f: &Box<AST>, args: &Vec<Box<AST>>) -> RValType {
    let lval = gen_lval(ctx, f, true);
    let (inst, ret) = match &lval {
        ValType::Method(mod_name, class, name) => {
            // TODO priavte and public

            // Find method
            let mod_ref = ctx.mgr.mod_tbl.get(mod_name).unwrap();
            let class_ref = mod_ref.get_class(class).unwrap();
            let m = class_ref.get_method(name).unwrap();
            let (flag, ps, ret_ty) = match m {
                MethodRef::Method(m) => (&m.flag, &m.ps, &m.ret),
                MethodRef::ExtMethod(m) => (&m.flag, &m.ps, &m.ret),
            };

            // Add to class file
            let sig = ctx.module.builder.borrow_mut().add_method_sig(
                !flag.is(MethodAttribFlag::Static),
                ps,
                ret_ty,
            );
            let (m_idx, tok_tag) = ctx
                .module
                .builder
                .borrow_mut()
                .add_const_member(mod_name, class, name, sig);

            build_args(ctx, ps, args);

            (
                if flag.is(MethodAttribFlag::Static) {
                    Inst::Call(to_tok(m_idx, tok_tag))
                } else {
                    let tok = to_tok(m_idx, tok_tag);
                    Inst::CallVirt(tok)
                },
                ret_ty.clone(),
            )
        }
        ValType::Module(_) => panic!(),
        ValType::Class(_, _) => panic!(),
        _ => unreachable!(),
    };

    ctx.method_builder.borrow_mut().add_inst(inst);

    ret
}

fn gen_new(ctx: &CodeGenCtx, ty: &Box<AST>, fields: &Vec<Box<AST>>) -> RValType {
    let ret = ctx.get_ty(ty);
    match &ret {
        RValType::Obj(mod_name, class_name) => {
            let class_ref = ctx
                .mgr
                .mod_tbl
                .get(mod_name)
                .unwrap()
                .get_class(class_name)
                .unwrap();
            let (_, class_instance_fields) = class_ref.get_info();

            let mut correct_idx: Vec<i32> = vec![-1; class_instance_fields.len()];
            for (i, field) in fields.iter().enumerate() {
                if let AST::StructExprField(field_name, _) = field.as_ref() {
                    let mut matched = false;
                    for (j, dec_field_name) in class_instance_fields.iter().enumerate() {
                        if dec_field_name == field_name {
                            correct_idx[j] = i as i32;
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        panic!("Class {} has no field {}", class_name, field_name);
                    }
                }
            }

            let mut ctor_ps: Vec<Param> = Vec::new();
            for (i, idx) in correct_idx.iter().enumerate() {
                if *idx < 0 {
                    panic!(
                        "{}.{} is not initialized in new expr",
                        class_name, class_instance_fields[i]
                    );
                }

                if let AST::StructExprField(field_name, expr) = fields[*idx as usize].as_ref() {
                    let v_ty = match expr.as_ref() {
                        AST::None => gen(ctx, &Box::new(AST::Id(field_name.to_owned()))),
                        _ => gen(ctx, expr),
                    }
                    .expect_rval();

                    if let Some(field) = class_ref.get_field(field_name) {
                        let field_ty = match field {
                            FieldRef::Field(f) => f.ty.clone(),
                            FieldRef::ExtField(f) => f.ty.clone(),
                        };
                        if field_ty != v_ty {
                            panic!(
                                "Cannot assign {} to {}.{}: {}",
                                v_ty, class_name, field_name, field_ty
                            );
                        }
                        ctor_ps.push(Param {
                            id: field_name.clone(),
                            attrib: ParamAttrib::default(),
                            ty: field_ty,
                        });
                    } else {
                        unreachable!();
                    }
                }
            }

            let mut builder = ctx.module.builder.borrow_mut();
            let ctor_sig = builder.add_method_sig(true, &ctor_ps, &RValType::Void);
            let (ctor_idx, tok_tag) =
                builder.add_const_member(mod_name, class_name, CTOR_NAME, ctor_sig);
            ctx.method_builder
                .borrow_mut()
                .add_inst(Inst::NewObj(to_tok(ctor_idx, tok_tag)));
        }
        RValType::Array(_) => unimplemented!(),
        _ => panic!("Invalid new expression, only new class or array is allowed"),
    }
    ret
}

fn gen_assign(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let lval = gen_lval(ctx, lhs, false);
    let v_ty = gen(ctx, rhs).expect_rval();

    match lval {
        ValType::Local(idx) => {
            let locals = ctx.locals.borrow();
            let local = &locals.locals[idx];
            let local_ty = local.ty.clone();

            if local_ty != v_ty {
                panic!("Cannot assign {} to local {}: {}", v_ty, local.id, local_ty);
            }

            ctx.method_builder.borrow_mut().add_inst_stloc(local.idx);
        }
        ValType::KwLSelf => {
            // lval guarentee that we are in instance method
            ctx.method_builder.borrow_mut().add_inst_starg(0);
        }
        ValType::Arg(idx) => {
            let arg = &ctx.method.ps[idx];

            if arg.ty != v_ty {
                panic!("Cannot assign {} to arg {}: {}", v_ty, arg.id, arg.ty);
            }

            ctx.method_builder.borrow_mut().add_inst_starg(if ctx
                .method
                .flag
                .is(MethodAttribFlag::Static)
            {
                idx
            } else {
                idx + 1
            } as u16);
        }
        ValType::Field(mod_name, class_name, name) => {
            // TODO private and public
            let (field_ty, field_flag) = match ctx
                .mgr
                .mod_tbl
                .get(&mod_name)
                .unwrap()
                .as_ref()
                .get_class(&class_name)
                .unwrap()
                .get_field(&name)
                .unwrap()
            {
                FieldRef::Field(f) => (f.ty.clone(), f.flag.clone()),
                FieldRef::ExtField(f) => (f.ty.clone(), f.flag.clone()),
            };

            if field_ty != v_ty {
                panic!(
                    "Cannot assign {} value to field {}/{}.{}: {}",
                    v_ty, mod_name, class_name, name, field_ty
                );
            }

            let sig = ctx.module.builder.borrow_mut().add_field_sig(&field_ty);
            let (f_idx, tok_tag) = ctx.module.builder.borrow_mut().add_const_member(
                &mod_name,
                &class_name,
                &name,
                sig,
            );
            let inst = if field_flag.is(FieldAttribFlag::Static) {
                Inst::StSFld(to_tok(f_idx, tok_tag))
            } else {
                Inst::StFld(to_tok(f_idx, tok_tag))
            };

            ctx.method_builder.borrow_mut().add_inst(inst);
        }
        ValType::Module(_) => panic!(),
        ValType::Class(_, _) => panic!(),
        _ => unreachable!(),
    }

    // assign op has no value left on evaluation stack
    RValType::Void
}

fn gen_numeric(ctx: &CodeGenCtx, op: BinOp, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let lty = gen(ctx, lhs).expect_rval();
    let rty = gen(ctx, rhs).expect_rval();

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

fn gen_cmp(ctx: &CodeGenCtx, op: BinOp, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let lty = gen(ctx, lhs).expect_rval();
    let rty = gen(ctx, rhs).expect_rval();

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

fn gen_id_rval(ctx: &CodeGenCtx, id: &str) -> RValType {
    // try search locals
    {
        let locals = ctx.locals.borrow();
        let is_instance_method = !ctx.method.flag.is(MethodAttribFlag::Static);
        if id == "self" {
            if is_instance_method {
                // first argument
                ctx.method_builder.borrow_mut().add_inst_ldarg(0);
                return RValType::Obj(ctx.module.fullname().to_owned(), ctx.class.name.clone());
            } else {
                panic!("Invalid self keyword in static method");
            }
        } else if let Some(local_var) = locals.get(id) {
            ctx.method_builder
                .borrow_mut()
                .add_inst_ldloc(local_var.idx);
            return local_var.ty.clone();
        } else if let Some(arg_idx) = ctx.method.ps_map.get(id) {
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
