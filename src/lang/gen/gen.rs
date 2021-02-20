use super::super::ast::AST;
use super::lval::gen_lval;
use super::{CodeGenCtx, LValType, ValType};
use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::ty::IrValType;

pub fn gen(ctx: &CodeGenCtx, ast: &Box<AST>) -> ValType {
    /*
    // TODO Use constant folding
    if ast.is_const() {
        gen(ctx, &Box::new(const_collapse(ast)))
    } else {

    }
    */
    match ast.as_ref() {
        AST::Block(children) => gen_block(ctx, children),
        AST::Stmt(stmt) => gen_stmt(ctx, stmt),
        AST::Let(pattern, flag, ty, init) => ValType::RVal(gen_let(ctx, pattern, flag, ty, init)),
        AST::OpNew(ty, fields) => ValType::RVal(gen_new(ctx, ty, fields)),
        AST::OpCall(f, args) => ValType::RVal(gen_call(ctx, f, args)),
        AST::OpAssign(lhs, rhs) => ValType::RVal(gen_assign(ctx, lhs, rhs)),
        AST::OpAdd(lhs, rhs) => ValType::RVal(gen_add(ctx, lhs, rhs)),
        AST::OpObjAccess(_, _) => {
            let lval = gen_lval(ctx, ast, false);
            match &lval {
                LValType::Field(mod_name, class_name, field_name) => {
                    let mod_rc = ctx.mgr.mod_tbl.get(mod_name).unwrap().upgrade().unwrap();
                    let class_ref = mod_rc.classes.get(class_name).unwrap().borrow();
                    let f = class_ref.fields.get(field_name).unwrap();

                    let ret = f.ty.clone();
                    let field_idx = ctx.module.builder.borrow_mut().add_const_fieldref(
                        mod_name,
                        class_name,
                        field_name,
                        &ret.descriptor(),
                    );
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst(Inst::LdFld(field_idx));

                    ValType::RVal(ret)
                }
                _ => unreachable!(),
            }
        }
        AST::OpStaticAccess(_, _) => {
            let lval = gen_lval(ctx, ast, false);
            match &lval {
                LValType::Field(mod_name, class_name, field_name) => {
                    let mod_rc = ctx.mgr.mod_tbl.get(mod_name).unwrap().upgrade().unwrap();
                    let class_ref = mod_rc.classes.get(class_name).unwrap().borrow();
                    let f = class_ref.fields.get(field_name).unwrap();

                    let ret = f.ty.clone();
                    let field_idx = ctx.module.builder.borrow_mut().add_const_fieldref(
                        mod_name,
                        class_name,
                        field_name,
                        &ret.descriptor(),
                    );
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst(Inst::LdSFld(field_idx));

                    ValType::RVal(ret)
                }
                _ => unreachable!(),
            }
        }
        AST::Id(id) => ValType::RVal(gen_id_rval(ctx, id)),
        AST::Int(val) => {
            ctx.method_builder.borrow_mut().add_inst_ldc(*val);
            ValType::RVal(IrValType::I32)
        }
        AST::None => ValType::RVal(IrValType::Void),
        _ => unimplemented!("{}", ast),
    }
}

fn gen_block(ctx: &CodeGenCtx, children: &Vec<Box<AST>>) -> ValType {
    // Push Symbol table
    ctx.locals.borrow_mut().push();

    let mut ret = ValType::RVal(IrValType::Void);
    for stmt in children.iter() {
        ret = gen(ctx, stmt);
    }

    // Pop Symbol table
    ctx.locals.borrow_mut().pop();
    ret
}

fn gen_stmt(ctx: &CodeGenCtx, stmt: &Box<AST>) -> ValType {
    let ret = gen(ctx, stmt);
    match &ret {
        ValType::RVal(ty) => {
            // pop from stack
            match ty {
                IrValType::Void => (),
                _ => ctx.method_builder.borrow_mut().add_inst(Inst::Pop),
            };
            ValType::RVal(IrValType::Void)
        }
        ValType::Ret(_) => ret,
        ValType::LVal(_) => unreachable!(),
    }
}

fn gen_let(
    ctx: &CodeGenCtx,
    pattern: &Box<AST>,
    flag: &LocalFlag,
    ty: &Box<AST>,
    init: &Box<AST>,
) -> IrValType {
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

    IrValType::Void
}

fn build_args(ctx: &CodeGenCtx, ps: &Vec<IrValType>, args: &Vec<Box<AST>>) {
    let args_ty: Vec<IrValType> = args.iter().map(|arg| gen(ctx, arg).expect_rval()).collect();

    for (i, (p_ty, arg_ty)) in ps.iter().zip(args_ty.iter()).enumerate() {
        if p_ty != arg_ty {
            panic!(
                "Call parameter type mismatch, expect {} but found {} at {}",
                p_ty, arg_ty, i
            );
        }
    }
}

fn gen_call(ctx: &CodeGenCtx, f: &Box<AST>, args: &Vec<Box<AST>>) -> IrValType {
    let lval = gen_lval(ctx, f, true);
    let (inst, ret) = match &lval {
        LValType::Method(mod_name, class, name) => {
            // TODO priavte and public

            // Find method
            let mod_rc = ctx.mgr.mod_tbl.get(mod_name).unwrap().upgrade().unwrap();
            let class_ref = mod_rc.classes.get(class).unwrap().borrow_mut();
            let m = class_ref.methods.get(name).unwrap();

            // Add to class file
            let m_idx = ctx.module.builder.borrow_mut().add_const_methodref(
                mod_name,
                class,
                name,
                &m.descriptor(),
            );
            let inst = if m.flag.is(MethodFlagTag::Static) {
                Inst::Call(m_idx)
            } else {
                Inst::CallVirt(m_idx)
            };

            build_args(ctx, &m.ps_ty, args);
            (inst, m.ret_ty.clone())
        }
        LValType::Module(_) => panic!(),
        LValType::Class(_, _) => panic!(),
        _ => unreachable!(),
    };

    ctx.method_builder.borrow_mut().add_inst(inst);

    ret
}

fn gen_new(ctx: &CodeGenCtx, ty: &Box<AST>, fields: &Vec<Box<AST>>) -> IrValType {
    let ret = ctx.get_ty(ty);
    match &ret {
        IrValType::Obj(mod_name, class_name) => {
            let mod_rc = ctx.mgr.mod_tbl.get(mod_name).unwrap().upgrade().unwrap();
            let class_ref = mod_rc.classes.get(class_name).unwrap().borrow();

            let mut correct_idx: Vec<i32> = vec![-1; class_ref.non_static_fields.len()];
            for (i, field) in fields.iter().enumerate() {
                if let AST::StructExprField(field_name, _) = field.as_ref() {
                    let mut matched = false;
                    for (j, dec_field_name) in class_ref.non_static_fields.iter().enumerate() {
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
            for (i, idx) in correct_idx.iter().enumerate() {
                if *idx < 0 {
                    panic!(
                        "{}.{} is not initialized in new expr",
                        class_name, class_ref.non_static_fields[i]
                    );
                }

                if let AST::StructExprField(field_name, expr) = fields[*idx as usize].as_ref() {
                    let v_ty = match expr.as_ref() {
                        AST::None => gen(ctx, &Box::new(AST::Id(field_name.to_owned()))),
                        _ => gen(ctx, expr),
                    }
                    .expect_rval();

                    if let Some(field) = class_ref.fields.get(field_name) {
                        if field.ty != v_ty {
                            panic!(
                                "Cannot assign {} to {}.{}: {}",
                                v_ty, class_name, field_name, field.ty
                            );
                        }
                    } else {
                        unreachable!();
                    }
                }
            }

            let class_idx = ctx
                .module
                .builder
                .borrow_mut()
                .add_const_class(mod_name, class_name);
            ctx.method_builder
                .borrow_mut()
                .add_inst(Inst::New(class_idx));
        }
        IrValType::Array(_) => unimplemented!(),
        _ => panic!("Invalid new expression, only new class or array is allowed"),
    }
    ret
}

fn gen_assign(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> IrValType {
    let lval = gen_lval(ctx, lhs, false);
    let v_ty = gen(ctx, rhs).expect_rval();

    match lval {
        LValType::Local(name) => {
            let locals = ctx.locals.borrow();
            let local = locals.get(&name).unwrap();
            let local_ty = local.ty.clone();

            if local_ty != v_ty {
                panic!("Cannot assign {} to local var {}: {}", v_ty, name, local_ty);
            }

            ctx.method_builder.borrow_mut().add_inst_stloc(local.offset);

            local_ty
        }
        LValType::Arg(name) => {
            let arg = ctx.args_map.get(&name).unwrap();

            if arg.ty != v_ty {
                panic!("Cannot assign {} to arg {}: {}", v_ty, name, arg.ty);
            }

            ctx.method_builder.borrow_mut().add_inst_starg(arg.offset);

            arg.ty.clone()
        }
        LValType::Field(mod_name, class_name, name) => {
            // TODO private and public
            let mod_rc = ctx.mgr.mod_tbl.get(&mod_name).unwrap().upgrade().unwrap();
            let class_ref = mod_rc.classes.get(&class_name).unwrap().borrow();
            let field = class_ref.fields.get(&name).unwrap();
            let field_ty = field.ty.clone();

            if field_ty != v_ty {
                panic!(
                    "Cannot assign {} value to field {}/{}.{}: {}",
                    v_ty, mod_name, class_name, name, field_ty
                );
            }

            let f_idx = ctx.module.builder.borrow_mut().add_const_fieldref(
                &mod_name,
                &class_name,
                &name,
                &field_ty.descriptor(),
            );
            let inst = if field.flag.is(FieldFlagTag::Static) {
                Inst::StSFld(f_idx)
            } else {
                Inst::StFld(f_idx)
            };

            ctx.method_builder.borrow_mut().add_inst(inst);
            field_ty
        }
        LValType::Module(_) => panic!(),
        LValType::Class(_, _) => panic!(),
        _ => unreachable!(),
    }
}

fn gen_add(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> IrValType {
    let lty = gen(ctx, lhs).expect_rval();
    let rty = gen(ctx, rhs).expect_rval();

    if lty != rty {
        panic!("Cannot add between {} and {}", lty, rty);
    }

    ctx.method_builder.borrow_mut().add_inst(Inst::Add);
    lty
}

fn gen_id_rval(ctx: &CodeGenCtx, id: &String) -> IrValType {
    // try search locals
    {
        let locals = ctx.locals.borrow();
        if let Some(local_var) = locals.get(id) {
            ctx.method_builder
                .borrow_mut()
                .add_inst_ldloc(local_var.offset);
            return local_var.ty.clone();
        } else if let Some(arg) = ctx.args_map.get(id) {
            ctx.method_builder.borrow_mut().add_inst_ldarg(arg.offset);
            return arg.ty.clone();
        }
    }
    unimplemented!("{} is not local nor arg", id);
}