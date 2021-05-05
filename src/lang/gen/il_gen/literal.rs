use super::super::{CodeGenCtx, RValType, ValType};

use xir::Inst;

pub fn gen_bool(ctx: &CodeGenCtx, val: bool) -> ValType {
    ctx.method_builder
        .borrow_mut()
        .add_inst_ldc(if val { 1 } else { 0 });
    ValType::RVal(RValType::Bool)
}

pub fn gen_int(ctx: &CodeGenCtx, val: i32) -> ValType {
    ctx.method_builder.borrow_mut().add_inst_ldc(val);
    ValType::RVal(RValType::I32)
}

pub fn gen_string(ctx: &CodeGenCtx, val: &str) -> ValType {
    let usr_str_heap_idx = ctx.module.builder.borrow_mut().add_const_usr_str(val);
    ctx.method_builder
        .borrow_mut()
        .add_inst(Inst::LdStr(usr_str_heap_idx));
    ValType::RVal(RValType::String)
}

pub fn gen_none() -> ValType {
    ValType::RVal(RValType::Void)
}
