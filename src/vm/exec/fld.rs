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
        | BuiltinType::SZArray(_) => stack.push_ptr(*(addr as *const *mut u8)),
    }
}

pub fn exec_ldfld(cur_ar: &mut ActivationRecord) {
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };

    let tok = cur_ar.consume_u32();
    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe { ctx.memberref[idx as usize - 1].expect_field().as_ref() },

        _ => unimplemented!(),
    };

    let instance_addr: *mut u8 = unsafe { cur_ar.eval_stack.pop_with_slot().expect_ref_or_ptr() };

    unsafe {
        do_load(
            instance_addr.wrapping_add(f.offset),
            &f.ty,
            &mut cur_ar.eval_stack,
        );
    }
}

pub fn exec_ldflda(cur_ar: &mut ActivationRecord) {
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };

    let tok = cur_ar.consume_u32();
    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe { ctx.memberref[idx as usize - 1].expect_field().as_ref() },

        _ => unimplemented!(),
    };

    let instance_addr_slot = cur_ar.eval_stack.pop_with_slot();
    let instance_addr: *mut u8 = unsafe { instance_addr_slot.expect_ref_or_ptr() };
    let fld_addr = instance_addr.wrapping_add(f.offset);

    if let SlotTag::INative = instance_addr_slot.tag {
        unimplemented!();
    } else {
        cur_ar.eval_stack.push_managed(fld_addr);
    }
}

pub fn exec_stfld(cur_ar: &mut ActivationRecord) {
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };

    let tok = cur_ar.consume_u32();
    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe { ctx.memberref[idx as usize - 1].expect_field().as_ref() },
        _ => unimplemented!(),
    };

    let v = cur_ar.eval_stack.pop_with_slot();

    let instance_addr: *mut u8 = unsafe { cur_ar.eval_stack.pop_with_slot().expect_ref() };
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
            | BuiltinType::SZArray(_) => {
                v.expect(SlotTag::Ref);
                *(field_addr as *mut *mut u8) = v.data.ptr_;
            }
        }
    }
}

pub fn exec_ldsfld(cur_ar: &mut ActivationRecord) {
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };

    let tok = cur_ar.consume_u32();
    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe { ctx.memberref[idx as usize - 1].expect_field().as_ref() },
        _ => unimplemented!(),
    };

    unsafe {
        do_load(f.addr, &f.ty, &mut cur_ar.eval_stack);
    }
}

pub fn exec_ldsflda(cur_ar: &mut ActivationRecord) {
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };

    let tok = cur_ar.consume_u32();
    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe { ctx.memberref[idx as usize - 1].expect_field().as_ref() },
        _ => unimplemented!(),
    };

    cur_ar.eval_stack.push_managed(f.addr);
}

pub fn exec_stsfld(cur_ar: &mut ActivationRecord) {
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };

    let tok = cur_ar.consume_u32();

    let (tag, idx) = get_tok_tag(tok);
    let f = match tag {
        TokTag::Field => ctx.fields[idx as usize - 1].as_ref(),
        TokTag::MemberRef => unsafe { ctx.memberref[idx as usize - 1].expect_field().as_ref() },
        _ => unimplemented!(),
    };

    let v = cur_ar.eval_stack.pop_with_slot();
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
            | BuiltinType::SZArray(_) => {
                v.expect(SlotTag::Ref);
                *(field_addr as *mut *mut u8) = v.data.ptr_;
            }
        }
    }
}
