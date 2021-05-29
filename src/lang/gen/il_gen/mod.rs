mod acc;
mod branch_expr;
mod call;
mod cast;
mod literal;
mod loop_expr;
mod op;

use super::super::ast::{ASTType, AST};
use op::BinOp;
// use super::interpreter::constant_folding;
use super::{CodeGenCtx, Field, Method, Module, RValType, SymType, Type, ValExpectation, ValType};

use xir::attrib::*;
use xir::inst::Inst;
use xir::tok::to_tok;
use xir::util::path::IModPath;
use xir::CTOR_NAME;

use std::ptr::NonNull;

pub fn gen_base_ctor(ctx: &CodeGenCtx, args: &Vec<Box<AST>>) {
    let base = unsafe { ctx.class.extends.as_ref().unwrap() };

    // similar to gen_new
    let ctors = base.methods.get(CTOR_NAME).unwrap();
    let args_ty: Vec<RValType> = args
        .iter()
        .map(|arg| gen(ctx, arg, ValExpectation::RVal).expect_rval())
        .collect();

    let ctor = call::pick_method_from_refs(ctors, &args_ty);
    let ctor = if let Some(ctor) = ctor {
        ctor
    } else {
        panic!("Cannot find ctor");
    };

    let mut builder = ctx.module.builder.borrow_mut();
    let ctor_sig = builder.add_method_sig(true, &ctor.ps, &RValType::Void);
    let (ctor_idx, tok_tag) = builder.add_const_member(
        unsafe { base.parent.as_ref().fullname() },
        &base.name,
        CTOR_NAME,
        ctor_sig,
    );

    ctx.method_builder.borrow_mut().add_inst(Inst::LdArg0); // load self
    ctx.method_builder
        .borrow_mut()
        .add_inst(Inst::Call(to_tok(ctor_idx, tok_tag)));
}

pub fn gen(ctx: &CodeGenCtx, ast: &AST, expectation: ValExpectation) -> ValType {
    /*
    if ctx.mgr.cfg.optim >= 1 && ast.is_constant() {
        return gen(ctx, &Box::new(constant_folding(ast)));
    }
    */

    match ast {
        AST::Block(children) => gen_block(ctx, children, expectation),
        AST::ExprStmt(stmt) => gen_expr_stmt(ctx, stmt),
        AST::If(cond, then, els) => {
            ValType::RVal(branch_expr::gen_if(ctx, cond, then, els, expectation))
        }
        AST::Let(pattern, flag, ty, init) => {
            gen_let(ctx, pattern, flag, ty, init);
            ValType::RVal(RValType::Void)
        }
        AST::Return(v) => {
            let ret = gen(ctx, v, ValExpectation::RVal).expect_rval();
            ctx.method_builder.borrow_mut().add_inst(Inst::Ret);
            ValType::Ret(ret)
        }
        AST::Loop(body) => ValType::RVal(loop_expr::gen_loop(ctx, body, expectation)),
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
        AST::OpObjAccess(lhs, rhs) => acc::gen_instance_acc(ctx, lhs, rhs, expectation),
        AST::OpStaticAccess(lhs, rhs) => acc::gen_static_acc(ctx, lhs, rhs, expectation),
        AST::OpArrayAccess(lhs, rhs) => acc::gen_arr_acc(ctx, lhs, rhs, expectation),
        AST::Id(id) => gen_id(ctx, id, expectation),
        AST::Type(ty) => match expectation {
            ValExpectation::Callable
            | ValExpectation::RVal
            | ValExpectation::Instance
            | ValExpectation::Assignable
            | ValExpectation::None => {
                panic!()
            }
            ValExpectation::Static => ValType::Sym(gen_type(ctx, ty)),
        },
        AST::Bool(val) => literal::gen_bool(ctx, *val),
        AST::Int(val) => literal::gen_int(ctx, *val),
        AST::String(val) => literal::gen_string(ctx, val),
        AST::None => literal::gen_none(),
        _ => unimplemented!("{}", ast),
    }
}

fn gen_block(ctx: &CodeGenCtx, children: &Vec<Box<AST>>, expectation: ValExpectation) -> ValType {
    // Push Symbol table
    ctx.locals.borrow_mut().push();

    let mut ret = ValType::RVal(RValType::Void);
    let mut child_iter = children.iter().peekable();
    while let Some(stmt) = child_iter.next() {
        if child_iter.peek().is_some() {
            // not last
            gen(ctx, stmt, ValExpectation::None);
            if ctx.method_builder.borrow().cur_bb_last_is_branch() {
                // do not generate unreachable stmts.
                // branch should be the last inst in bb
                break;
            }
        } else {
            // last
            ret = gen(ctx, stmt, expectation);
        }
    }

    // Pop Symbol table
    ctx.locals.borrow_mut().pop();
    ret
}

fn gen_expr_stmt(ctx: &CodeGenCtx, stmt: &Box<AST>) -> ValType {
    let ret = gen(ctx, stmt, ValExpectation::RVal);
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

fn gen_let(ctx: &CodeGenCtx, pattern: &Box<AST>, flag: &LocalAttrib, ty: &ASTType, init: &AST) {
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
                let init_ty = gen(ctx, init, ValExpectation::RVal).expect_rval();
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
}

fn gen_assign(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    // filter rval value type by expect_sym
    let lval = gen(ctx, lhs, ValExpectation::Assignable).expect_sym();
    let v_ty = gen(ctx, rhs, ValExpectation::RVal).expect_rval();

    match &lval {
        SymType::Local(idx) => {
            let locals = ctx.locals.borrow();
            let local = &locals.locals[*idx];
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
            let arg = &ctx.method.ps[*idx];

            if arg.ty != v_ty {
                panic!("Cannot assign {} to arg {}: {}", v_ty, arg.id, arg.ty);
            }

            ctx.method_builder.borrow_mut().add_inst_starg(if ctx
                .method
                .attrib
                .is(MethodAttribFlag::Static)
            {
                *idx
            } else {
                *idx + 1
            } as u16);
        }
        SymType::Field(f) => {
            // TODO private and public
            let (mod_name, class_name, f_ref) = unsafe {
                let f_ref = f.as_ref();
                let class_ref = f_ref.parent.as_ref();
                let module_ref = class_ref.parent.as_ref();
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
            if ele_ty != &v_ty {
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

fn gen_id(ctx: &CodeGenCtx, id: &str, expectation: ValExpectation) -> ValType {
    // try search locals
    match expectation {
        ValExpectation::None | ValExpectation::Callable => {
            // currently only method is callable
            let ms = ctx.class.query_method(id);
            if ms.is_empty() {
                panic!(
                    "No method {} in class {}/{}",
                    id,
                    ctx.module.get_module().fullname(),
                    ctx.class.name
                );
            }
            ValType::Sym(SymType::Method(
                ms.into_iter()
                    .map(|m| NonNull::new(m as *const Method as *mut Method).unwrap())
                    .collect(),
            ))
        }
        ValExpectation::RVal | ValExpectation::Instance => {
            let locals = ctx.locals.borrow();
            let is_instance_method = !ctx.method.attrib.is(MethodAttribFlag::Static);
            let mut loada = false;
            ValType::RVal(if id == "self" {
                if is_instance_method {
                    // first argument
                    ctx.method_builder.borrow_mut().add_inst_ldarg(0);
                    RValType::Type(NonNull::new(ctx.class as *const Type as *mut Type).unwrap())
                } else {
                    panic!("Invalid self keyword in static method");
                }
            } else if let Some(local_var) = locals.get(id) {
                if let ValExpectation::Instance = expectation {
                    if let RValType::Type(_ty) = local_var.ty {
                        if unsafe { _ty.as_ref() }.is_struct() {
                            loada = true;
                        }
                    }
                }

                if loada {
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst_ldloca(local_var.idx);
                    RValType::ByRef(Box::new(local_var.ty.clone()))
                } else {
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst_ldloc(local_var.idx);
                    local_var.ty.clone()
                }
            } else if let Some(arg_idx) = ctx.ps_map.get(id) {
                let arg = &ctx.method.ps[*arg_idx];
                if let ValExpectation::Instance = expectation {
                    if let RValType::Type(_ty) = arg.ty {
                        if unsafe { _ty.as_ref() }.is_struct() {
                            loada = true;
                        }
                    }
                }
                if loada {
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst_ldarga(if is_instance_method {
                            *arg_idx + 1
                        } else {
                            *arg_idx
                        } as u16);
                    RValType::ByRef(Box::new(arg.ty.clone()))
                } else {
                    ctx.method_builder
                        .borrow_mut()
                        .add_inst_ldarg(if is_instance_method {
                            *arg_idx + 1
                        } else {
                            *arg_idx
                        } as u16);
                    arg.ty.clone()
                }
            } else {
                panic!();
            })
        }
        ValExpectation::Static => {
            let id = if id == "Self" { &ctx.class.name } else { id };
            ValType::Sym(if let Some(path) = ctx.module.use_map.get(id) {
                // item in sub module or any using module
                SymType::Module(
                    NonNull::new(ctx.mgr.mod_tbl.get(path.as_str()).unwrap().as_ref()
                        as *const Module as *mut Module)
                    .unwrap(),
                )
            } else if ctx.module.get_module().sub_mods.contains(id) {
                // a submodule in this module
                let mut path = ctx.module.get_module().mod_path.clone();
                path.push(id);
                SymType::Module(
                    NonNull::new(ctx.mgr.mod_tbl.get(path.as_str()).unwrap().as_ref()
                        as *const Module as *mut Module)
                    .unwrap(),
                )
            } else if let Some(c) = ctx.module.get_module().classes.get(id) {
                // class within the same module
                SymType::Class(NonNull::new(c.as_ref() as *const Type as *mut Type).unwrap())
            } else if let Some(m) = ctx.mgr.mod_tbl.get(id) {
                // this crate can be referenced in this case (allow or not?)
                SymType::Module(NonNull::new(m.as_ref() as *const Module as *mut Module).unwrap())
            } else {
                panic!();
            })
        }
        ValExpectation::Assignable => {
            let is_instance_method = !ctx.method.attrib.is(MethodAttribFlag::Static);
            ValType::Sym(if id == "self" {
                if is_instance_method {
                    SymType::KwLSelf
                } else {
                    panic!("invalid keyword self in static method");
                }
            } else if let Some(var) = ctx.locals.borrow().get(id) {
                // query local var
                SymType::Local(var.idx as usize)
            } else if let Some(arg) = ctx.ps_map.get(id) {
                // query args
                SymType::Arg(*arg)
            } else if let Some(f) = ctx.class.query_field(id) {
                // query field in this class
                // either static or non-static is ok
                SymType::Field(NonNull::new(f as *const Field as *mut Field).unwrap())
            } else {
                panic!("Cannot found item with id: {}", id);
            })
        }
    }
}

fn gen_type(ctx: &CodeGenCtx, ty: &ASTType) -> SymType {
    match ty {
        ASTType::Bool => unimplemented!(),
        ASTType::Char => unimplemented!(),
        ASTType::I32 => unimplemented!(),
        ASTType::F64 => unimplemented!(),
        ASTType::String => unimplemented!(),
        ASTType::Tuple(_) => unimplemented!(),
        ASTType::Arr(_) => unimplemented!(),
        ASTType::Class(path) => {
            assert!(path.len() == 1, "invalid path in lval gen \"{}\"", path);
            match path.as_str() {
                "Self" => SymType::Class(
                    NonNull::new(
                        ctx.module
                            .get_module()
                            .classes
                            .get(&ctx.class.name)
                            .unwrap()
                            .as_ref() as *const Type as *mut Type,
                    )
                    .unwrap(),
                ),
                _ => unreachable!(),
            }
        }
        ASTType::None => panic!(),
    }
}
