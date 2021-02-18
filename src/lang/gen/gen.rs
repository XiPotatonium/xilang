use super::super::ast::ast::AST;
use super::ctx::CodeGenCtx;
use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::ty::RValType;

pub enum ValType {
    LVal(LValType),
    RVal(RValType),
    Ret(RValType),
}

impl std::fmt::Display for ValType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LVal(lval) => unimplemented!(),
            Self::RVal(rval) => write!(f, "{} (RVal)", rval),
            Self::Ret(retv) => write!(f, "{} (Ret)", retv),
        }
    }
}

impl ValType {
    pub fn expect_rval(self) -> RValType {
        match self {
            Self::LVal(_) => panic!("Expect rval but found lval"),
            Self::Ret(_) => panic!("Expect rval but found return value"),
            Self::RVal(val) => val,
        }
    }
}

pub enum LValType {
    // class full name, method name
    Method(String, String),
    // class full name, field name
    Field(String, String),
    // class full name
    Class(String),
    // module full name
    Module(String),
    // local name
    Local(String),
    // Param name
    Arg(String),
}

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
                LValType::Field(class_name, field_name) => {
                    let class_rc = ctx
                        .mgr
                        .class_table
                        .get(class_name)
                        .unwrap()
                        .upgrade()
                        .unwrap();
                    let class_ref = class_rc.borrow();
                    let f = class_ref.fields.get(field_name).unwrap();

                    let ret = f.ty.clone();
                    let mut builder = ctx.class.builder.borrow_mut();
                    let field_idx =
                        builder.add_const_fieldref(class_name, field_name, &ret.descriptor());
                    builder.add_inst(ctx.method.method_idx, Inst::LdFld(field_idx));

                    ValType::RVal(ret)
                }
                _ => unreachable!(),
            }
        }
        AST::OpStaticAccess(_, _) => {
            let lval = gen_lval(ctx, ast, false);
            match &lval {
                LValType::Field(class_name, field_name) => {
                    let class_rc = ctx
                        .mgr
                        .class_table
                        .get(class_name)
                        .unwrap()
                        .upgrade()
                        .unwrap();
                    let class_ref = class_rc.borrow();
                    let f = class_ref.fields.get(field_name).unwrap();

                    let ret = f.ty.clone();
                    let mut builder = ctx.class.builder.borrow_mut();
                    let field_idx =
                        builder.add_const_fieldref(class_name, field_name, &ret.descriptor());
                    builder.add_inst(ctx.method.method_idx, Inst::LdSFld(field_idx));

                    ValType::RVal(ret)
                }
                _ => unreachable!(),
            }
        }
        AST::Id(id) => ValType::RVal(gen_id_rval(ctx, id)),
        AST::Int(val) => {
            ctx.class
                .builder
                .borrow_mut()
                .add_inst_ldc(ctx.method.method_idx, *val);
            ValType::RVal(RValType::I32)
        }
        AST::None => ValType::RVal(RValType::Void),
        _ => unimplemented!("{}", ast),
    }
}

fn gen_lval(ctx: &CodeGenCtx, ast: &Box<AST>, expect_method: bool) -> LValType {
    match ast.as_ref() {
        AST::Id(id) => {
            if expect_method {
                // query method in this class
                if ctx.class.methods.contains_key(id) {
                    return LValType::Method(ctx.class.fullname.clone(), id.to_owned());
                }
            } else {
                // query local var
                if ctx.locals.borrow().contains_key(id) {
                    return LValType::Local(id.to_owned());
                } else if ctx.args_map.contains_key(id) {
                    return LValType::Arg(id.to_owned());
                } else if ctx.class.fields.contains_key(id) {
                    // query field in this class
                    // either static or non-static is ok
                    return LValType::Field(ctx.class.fullname.clone(), id.to_owned());
                }
            }

            // module or class
            let class_in_cur_module = format!(
                "{}/{}",
                ctx.class.path[..ctx.class.path.len() - 1].join("/"),
                id
            );
            if ctx.mgr.class_table.contains_key(&class_in_cur_module) {
                // a class in current module
                LValType::Class(class_in_cur_module)
            } else {
                unimplemented!("Query {} as module not implemented", id);
            }
        }
        AST::OpObjAccess(lhs, rhs) => {
            // generate lhs as lval (as the first arg in non-static method or objectref of putfield)
            let lhs = gen(ctx, lhs).expect_rval();
            match lhs {
                RValType::Obj(name) => {
                    // Access a non-static method or non-static field in class
                    let class_rc = ctx.mgr.class_table.get(&name).unwrap().upgrade().unwrap();
                    let class_ref = class_rc.borrow();
                    if expect_method {
                        if let Some(m) = class_ref.methods.get(rhs) {
                            if m.flag.is(MethodFlagTag::Static) {
                                panic!("Cannot obj access static method {}::{}", name, rhs);
                            } else {
                                LValType::Method(name, rhs.to_owned())
                            }
                        } else {
                            panic!("No method {} found in class {}", rhs, name);
                        }
                    } else {
                        if let Some(f) = class_ref.fields.get(rhs) {
                            if f.flag.is(FieldFlagTag::Static) {
                                panic!("Cannot obj access static filed {}::{}", name, rhs);
                            } else {
                                LValType::Field(name, rhs.to_owned())
                            }
                        } else {
                            panic!("No field {} found in class {}", rhs, name);
                        }
                    }
                }
                _ => panic!("Cannot obj access a non-obj value"),
            }
        }
        AST::OpStaticAccess(lhs, rhs) => {
            let lhs = gen_lval(ctx, lhs, expect_method);
            match lhs {
                LValType::Module(name) => {
                    // Access a class or sub-class in module
                    unimplemented!();
                }
                LValType::Class(name) => {
                    // Access a static method or static field in class
                    let class_rc = ctx.mgr.class_table.get(&name).unwrap().upgrade().unwrap();
                    let class_ref = class_rc.borrow();
                    if expect_method {
                        if let Some(m) = class_ref.methods.get(rhs) {
                            if m.flag.is(MethodFlagTag::Static) {
                                LValType::Method(name, rhs.to_owned())
                            } else {
                                panic!("Cannot static access non-static method {}.{}", name, rhs);
                            }
                        } else {
                            panic!("No method {} found in class {}", rhs, name);
                        }
                    } else {
                        if let Some(f) = class_ref.fields.get(rhs) {
                            if f.flag.is(FieldFlagTag::Static) {
                                LValType::Field(name, rhs.to_owned())
                            } else {
                                panic!("Cannot static access non-static filed {}.{}", name, rhs);
                            }
                        } else {
                            panic!("No field {} found in class {}", rhs, name);
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
        AST::OpArrayAccess(lhs, rhs) => {
            unimplemented!();
        }
        _ => unimplemented!(),
    }
}

fn gen_block(ctx: &CodeGenCtx, children: &Vec<Box<AST>>) -> ValType {
    // Push Symbol table
    ctx.locals.borrow_mut().push();

    let mut ret = ValType::RVal(RValType::Void);
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
                RValType::Void => (),
                _ => ctx
                    .class
                    .builder
                    .borrow_mut()
                    .add_inst(ctx.method.method_idx, Inst::Pop),
            };
            ValType::RVal(RValType::Void)
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
                    let ty = ctx.class.get_type(ty, ctx.mgr);
                    ctx.locals.borrow_mut().add(id, ty.clone(), *flag, false);
                }
            } else {
                // build init
                let init_ty = gen(ctx, init).expect_rval();
                let offset = ctx
                    .locals
                    .borrow_mut()
                    .add(id, init_ty.clone(), *flag, true);
                ctx.class
                    .builder
                    .borrow_mut()
                    .add_inst_stloc(ctx.method.method_idx, offset);

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

    RValType::Void
}

fn build_args(ctx: &CodeGenCtx, ps: &Vec<RValType>, args: &Vec<Box<AST>>) {
    let args_ty: Vec<RValType> = args.iter().map(|arg| gen(ctx, arg).expect_rval()).collect();

    for (i, (p_ty, arg_ty)) in ps.iter().zip(args_ty.iter()).enumerate() {
        if p_ty != arg_ty {
            panic!(
                "Call parameter type mismatch, expect {} but found {} at {}",
                p_ty, arg_ty, i
            );
        }
    }
}

fn gen_call(ctx: &CodeGenCtx, f: &Box<AST>, args: &Vec<Box<AST>>) -> RValType {
    let lval = gen_lval(ctx, f, true);
    let (inst, ret) = match &lval {
        LValType::Method(class, name) => {
            // TODO priavte and public

            // Find method
            let class_rc = ctx.mgr.class_table.get(class).unwrap().upgrade().unwrap();
            let class_ref = class_rc.borrow();
            let m = class_ref.methods.get(name).unwrap();

            // Add to class file
            let m_idx =
                ctx.class
                    .builder
                    .borrow_mut()
                    .add_const_methodref(class, name, &m.descriptor());
            let inst = if m.flag.is(MethodFlagTag::Static) {
                Inst::Call(m_idx)
            } else {
                Inst::CallVirt(m_idx)
            };

            build_args(ctx, &m.ps_ty, args);
            (inst, m.ret_ty.clone())
        }
        LValType::Module(_) => panic!(),
        LValType::Class(_) => panic!(),
        _ => unreachable!(),
    };

    ctx.class
        .builder
        .borrow_mut()
        .add_inst(ctx.method.method_idx, inst);

    ret
}

fn gen_new(ctx: &CodeGenCtx, ty: &Box<AST>, fields: &Vec<Box<AST>>) -> RValType {
    let ret = ctx.class.get_type(ty, ctx.mgr);
    match &ret {
        RValType::Obj(class_name) => {
            let class = ctx
                .mgr
                .class_table
                .get(class_name)
                .unwrap()
                .upgrade()
                .unwrap();
            let class_ref = class.borrow();

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

            let class_idx = ctx.class.builder.borrow_mut().add_const_class(class_name);
            ctx.class
                .builder
                .borrow_mut()
                .add_inst(ctx.method.method_idx, Inst::New(class_idx));
        }
        RValType::Array(inner_ty) => unimplemented!(),
        _ => panic!("Invalid new expression, only new class or array is allowed"),
    }
    ret
}

fn gen_assign(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
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

            ctx.class
                .builder
                .borrow_mut()
                .add_inst_stloc(ctx.method.method_idx, local.offset);

            local_ty
        }
        LValType::Arg(name) => {
            let arg = ctx.args_map.get(&name).unwrap();

            if arg.ty != v_ty {
                panic!("Cannot assign {} to arg {}: {}", v_ty, name, arg.ty);
            }

            ctx.class
                .builder
                .borrow_mut()
                .add_inst_starg(ctx.method.method_idx, arg.offset);

            arg.ty.clone()
        }
        LValType::Field(class, name) => {
            // TODO private and public
            let class_rc = ctx.mgr.class_table.get(&class).unwrap().upgrade().unwrap();
            let class_ref = class_rc.borrow();
            let field = class_ref.fields.get(&name).unwrap();
            let field_ty = field.ty.clone();

            if field_ty != v_ty {
                panic!(
                    "Cannot assign {} value to field {}.{}: {}",
                    v_ty, class, name, field_ty
                );
            }

            let mut builder = ctx.class.builder.borrow_mut();
            let f_idx = builder.add_const_fieldref(&class, &name, &field_ty.descriptor());
            let inst = if field.flag.is(FieldFlagTag::Static) {
                Inst::StSFld(f_idx)
            } else {
                Inst::StFld(f_idx)
            };

            builder.add_inst(ctx.method.method_idx, inst);
            field_ty
        }
        LValType::Module(_) => panic!(),
        LValType::Class(_) => panic!(),
        _ => unreachable!(),
    }
}

fn gen_add(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> RValType {
    let lty = gen(ctx, lhs).expect_rval();
    let rty = gen(ctx, rhs).expect_rval();

    if lty != rty {
        panic!("Cannot add between {} and {}", lty, rty);
    }

    ctx.class
        .builder
        .borrow_mut()
        .add_inst(ctx.method.method_idx, Inst::Add);
    lty
}

fn gen_id_rval(ctx: &CodeGenCtx, id: &String) -> RValType {
    // try search locals
    {
        let locals = ctx.locals.borrow();
        if let Some(local_var) = locals.get(id) {
            ctx.class
                .builder
                .borrow_mut()
                .add_inst_ldloc(ctx.method.method_idx, local_var.offset);
            return local_var.ty.clone();
        } else if let Some(arg) = ctx.args_map.get(id) {
            ctx.class
                .builder
                .borrow_mut()
                .add_inst_ldarg(ctx.method.method_idx, arg.offset);
            return arg.ty.clone();
        }
    }
    unimplemented!("{} is not local nor arg", id);
}
