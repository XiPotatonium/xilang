use super::super::ast::ast::AST;
use super::ctx::CodeGenCtx;
use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::ty::{fn_descriptor, VarType, XirType};

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

fn gen_compound_lval(ctx: &CodeGenCtx, ast: &Box<AST>, prefer_field: bool) -> XirType {
    match ast.as_ref() {
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

fn query_method(ctx: &CodeGenCtx, id: &String) -> (u16, VarType, Vec<VarType>) {
    let locals = ctx.locals.borrow();
    let fields = ctx.class.fields.borrow();
    let methods = ctx.class.methods.borrow();
    let f: u16;
    let ret: VarType;
    let ps: Vec<VarType>;

    // query local var
    if let Some(_) = locals.get(id) {
        unimplemented!("Call a local variable is not implemented");
    }

    // query method in this class
    if let Some(m) = methods.get(id) {
        if m.flag.is(FlagTag::Static) {
            f = ctx.class.builder.borrow_mut().add_const_methodref(
                &ctx.class.fullname(),
                id,
                &fn_descriptor(&m.ret_ty, &m.ps_ty),
            );
            ret = m.ret_ty.clone();
            ps = m.ps_ty.to_vec();
            return (f, ret, ps);
        } else {
            panic!("Called method is not a static method: try self.{}()", id);
        }
    }

    // query field in this class
    if let Some(_) = fields.get(id) {
        unimplemented!("Call a field is not implemented");
    } else {
        panic!("No valid method found");
    }
}

fn gen_call(ctx: &CodeGenCtx, f: &Box<AST>, args: &Vec<Box<AST>>) -> VarType {
    let (f, ret, ps) = match f.as_ref() {
        AST::Id(id) => {
            // local or static field or static method
            query_method(ctx, id)
        }
        _ => {
            let lval = gen_compound_lval(ctx, f, false);
            match &lval {
                XirType::RVal(_) => unreachable!(),
                XirType::Method(class, id) => {
                    unimplemented!();
                }
                XirType::Field(_, _) => unimplemented!(),
                _ => panic!(),
            }
        }
    };

    let args_ty: Vec<VarType> = args.iter().map(|arg| gen(ctx, arg)).collect();

    for (i, (p_ty, arg_ty)) in ps.iter().zip(args_ty.iter()).enumerate() {
        if p_ty != arg_ty {
            panic!(
                "Call parameter type mismatch, expect {} but found {} at {}",
                p_ty, arg_ty, i
            );
        }
    }

    ret
}

fn gen_new(ctx: &CodeGenCtx, ty: &Box<AST>, fields: &Vec<Box<AST>>) -> VarType {
    let ret = ctx.class.get_type(ty, ctx.mgr);
    match &ret {
        VarType::Class(class_name) => {
            let class = ctx
                .mgr
                .class_table
                .get(class_name)
                .unwrap()
                .upgrade()
                .unwrap();
            for field in fields.iter() {
                if let AST::StructExprField(field_name, field_expr) = field.as_ref() {
                    unimplemented!();
                }
            }
            let class_name = ctx.class.builder.borrow_mut().add_const_class(class_name);
            ctx.class
                .builder
                .borrow_mut()
                .add_inst(ctx.method.method_idx, Inst::New(class_name));
        }
        VarType::Array(inner_ty) => unimplemented!(),
        _ => panic!("Invalid new expression, only new class or array is allowed"),
    }
    ret
}

fn gen_assign(ctx: &CodeGenCtx, lhs: &Box<AST>, rhs: &Box<AST>) -> VarType {
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
