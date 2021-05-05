use xir::tok::{get_tok_tag, TokTag};

use super::super::data::BuiltinType;
use super::super::stack::{ActivationRecord, EvalStack, SlotTag};

unsafe fn do_load(addr: *const u8, ty: &BuiltinType, stack: &mut EvalStack) {
    match ty {
        BuiltinType::Void | BuiltinType::Unk => {
            unreachable!()
        }
        BuiltinType::Bool => unimplemented!(),
        BuiltinType::Char => unimplemented!(),
        BuiltinType::U1 => unimplemented!(),
        BuiltinType::I1 => unimplemented!(),
        BuiltinType::U4 => unimplemented!(),
        BuiltinType::I4 => {
            stack.push_i32(*(addr as *const i32));
        }
        BuiltinType::U8 => unimplemented!(),
        BuiltinType::I8 => unimplemented!(),
        BuiltinType::UNative => unimplemented!(),
        BuiltinType::INative => unimplemented!(),
        BuiltinType::R4 => unimplemented!(),
        BuiltinType::R8 => unimplemented!(),
        BuiltinType::String
        | BuiltinType::Class(_)
        | BuiltinType::ByRef(_)
        | BuiltinType::Array(_) => stack.push_ptr(*(addr as *const *mut u8)),
    }
}

pub fn ldfld(cur_state: &mut ActivationRecord) {
    let ctx = unsafe { cur_state.method.ctx.as_ref().unwrap().expect_il() };

    let tok = cur_state.consume_u32();
    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe {
            ctx.memberref[idx as usize - 1]
                .expect_field()
                .as_ref()
                .unwrap()
        },

        _ => unimplemented!(),
    };

    let instance_addr: *mut u8 = unsafe { cur_state.eval_stack.pop_with_slot().as_addr() };
    unsafe {
        do_load(
            instance_addr.wrapping_add(f.offset),
            &f.ty,
            &mut cur_state.eval_stack,
        );
    }
}

pub fn stfld(cur_state: &mut ActivationRecord) {
    let ctx = unsafe { cur_state.method.ctx.as_ref().unwrap().expect_il() };

    let tok = cur_state.consume_u32();
    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe {
            ctx.memberref[idx as usize - 1]
                .expect_field()
                .as_ref()
                .unwrap()
        },
        _ => unimplemented!(),
    };

    let v = cur_state.eval_stack.pop_with_slot();

    let instance_addr: *mut u8 = unsafe { cur_state.eval_stack.pop_with_slot().as_addr() };
    let field_addr = instance_addr.wrapping_add(f.offset);

    unsafe {
        match f.ty {
            BuiltinType::Void | BuiltinType::Unk => {
                unreachable!()
            }
            BuiltinType::Bool => unimplemented!(),
            BuiltinType::Char => unimplemented!(),
            BuiltinType::U1 => unimplemented!(),
            BuiltinType::I1 => unimplemented!(),
            BuiltinType::U4 => unimplemented!(),
            BuiltinType::I4 => {
                v.expect(SlotTag::I32);
                *(field_addr as *mut i32) = v.data.i32_;
            }
            BuiltinType::U8 => unimplemented!(),
            BuiltinType::I8 => unimplemented!(),
            BuiltinType::UNative => unimplemented!(),
            BuiltinType::INative => unimplemented!(),
            BuiltinType::R4 => unimplemented!(),
            BuiltinType::R8 => unimplemented!(),
            BuiltinType::String
            | BuiltinType::Class(_)
            | BuiltinType::ByRef(_)
            | BuiltinType::Array(_) => {
                v.expect(SlotTag::Ref);
                *(field_addr as *mut *mut u8) = v.data.ptr_;
            }
        }
    }
}

pub fn ldsfld(cur_state: &mut ActivationRecord) {
    let ctx = unsafe { cur_state.method.ctx.as_ref().unwrap().expect_il() };

    let tok = cur_state.consume_u32();
    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe {
            ctx.memberref[idx as usize - 1]
                .expect_field()
                .as_ref()
                .unwrap()
        },
        _ => unimplemented!(),
    };

    unsafe {
        do_load(f.addr, &f.ty, &mut cur_state.eval_stack);
    }
}

pub fn stsfld(cur_state: &mut ActivationRecord) {
    let ctx = unsafe { cur_state.method.ctx.as_ref().unwrap().expect_il() };

    let tok = cur_state.consume_u32();

    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe {
            ctx.memberref[idx as usize - 1]
                .expect_field()
                .as_ref()
                .unwrap()
        },
        _ => unimplemented!(),
    };

    let v = cur_state.eval_stack.pop_with_slot();
    let field_addr = f.addr;

    unsafe {
        match f.ty {
            BuiltinType::Void | BuiltinType::Unk => {
                unreachable!()
            }
            BuiltinType::Bool => unimplemented!(),
            BuiltinType::Char => unimplemented!(),
            BuiltinType::U1 => unimplemented!(),
            BuiltinType::I1 => unimplemented!(),
            BuiltinType::U4 => unimplemented!(),
            BuiltinType::I4 => {
                v.expect(SlotTag::I32);
                *(field_addr as *mut i32) = v.data.i32_;
            }
            BuiltinType::U8 => unimplemented!(),
            BuiltinType::I8 => unimplemented!(),
            BuiltinType::UNative => unimplemented!(),
            BuiltinType::INative => unimplemented!(),
            BuiltinType::R4 => unimplemented!(),
            BuiltinType::R8 => unimplemented!(),
            BuiltinType::String
            | BuiltinType::Class(_)
            | BuiltinType::ByRef(_)
            | BuiltinType::Array(_) => {
                v.expect(SlotTag::Ref);
                *(field_addr as *mut *mut u8) = v.data.ptr_;
            }
        }
    }
}
