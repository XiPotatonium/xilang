use super::super::{CodeGenCtx, RValType, ValType};

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

pub fn gen_none() -> ValType {
    ValType::RVal(RValType::Void)
}
