use super::super::data::{Type, TypedAddr, I4_SIZE, REF_SIZE};
use super::super::heap::Heap;
use super::super::shared_mem::SharedMem;
use super::super::stack::{ActivationRecord, Slot, SlotTag};
use super::super::util::ptr::NonNull;

use xir::tok::{get_tok_tag, TokTag};

fn to_arr_size(slot: &Slot) -> isize {
    unsafe {
        match slot.tag {
            SlotTag::I32 => slot.data.i32_ as isize,
            SlotTag::INative => slot.data.inative_,
            _ => panic!(
                "new arr size should be i32 or inative but found {}",
                slot.tag
            ),
        }
    }
}

pub fn exec_newarr(cur_ar: &mut ActivationRecord, mem: &mut SharedMem) {
    let ty_tok = cur_ar.consume_u32();
    let (tok_tag, tok_idx) = get_tok_tag(ty_tok);
    let tok_idx = tok_idx as usize - 1;
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };
    let ele_ty = match tok_tag {
        TokTag::TypeDef => ctx.types[tok_idx].as_ref() as *const Type,
        TokTag::TypeRef => ctx.typerefs[tok_idx].as_ptr() as *const Type,
        TokTag::TypeSpec => {
            unimplemented!()
        }
        _ => unreachable!(),
    };

    let size = to_arr_size(&cur_ar.eval_stack.pop(None));

    let addr = unsafe { mem.new_arr(ele_ty, size as usize) };
    cur_ar.eval_stack.push_ptr(addr);
}

pub fn exec_ldlen(cur_ar: &mut ActivationRecord) {
    let arr = cur_ar.eval_stack.pop(None);
    let arr = unsafe { arr.expect_ref() };

    let len = Heap::get_arr_len(arr);
    cur_ar.eval_stack.push_usize(len);
}

pub fn exec_ldelem(_: &mut ActivationRecord) {
    unimplemented!("ldelem is not implemented");
}

pub fn exec_stelem(cur_ar: &mut ActivationRecord) {
    let ty_tok = cur_ar.consume_u32();
    let (tok_tag, tok_idx) = get_tok_tag(ty_tok);
    let tok_idx = tok_idx as usize - 1;
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };
    let ele_ty = match tok_tag {
        TokTag::TypeDef => ctx.types[tok_idx].as_ref(),
        TokTag::TypeRef => unsafe { ctx.typerefs[tok_idx].as_ref() },
        TokTag::TypeSpec => {
            unimplemented!()
        }
        _ => unreachable!(),
    };

    let ele_size = if ele_ty.ee_class.is_value {
        ele_ty.basic_instance_size
    } else {
        REF_SIZE
    };

    let idx = to_arr_size(cur_ar.eval_stack.peek_at(1).unwrap());
    let addr = unsafe { cur_ar.eval_stack.peek_at(2).unwrap().expect_ref() };
    let addr = Heap::get_arr_offset(addr, ele_size, idx as usize);
    cur_ar.eval_stack.pop(Some(TypedAddr {
        ty: NonNull::new(ele_ty as *const Type as *mut Type).unwrap(),
        addr,
    }));
    cur_ar.eval_stack.pop(None); // pop idx
    cur_ar.eval_stack.pop(None); // pop addr
}

pub fn exec_ldelema(cur_ar: &mut ActivationRecord) {
    let ty_tok = cur_ar.consume_u32();
    let (tok_tag, tok_idx) = get_tok_tag(ty_tok);
    let tok_idx = tok_idx as usize - 1;
    let ctx = unsafe { cur_ar.method.ctx.as_ref().expect_il() };
    let ele_ty = match tok_tag {
        TokTag::TypeDef => ctx.types[tok_idx].as_ref(),
        TokTag::TypeRef => unsafe { ctx.typerefs[tok_idx].as_ref() },
        TokTag::TypeSpec => {
            unimplemented!()
        }
        _ => unreachable!(),
    };

    let idx = to_arr_size(&cur_ar.eval_stack.pop(None));
    let arr = unsafe { cur_ar.eval_stack.pop(None).expect_ref() };
    if ele_ty.ee_class.is_value {
        let addr = Heap::get_arr_offset(arr, ele_ty.basic_instance_size, idx as usize);
        cur_ar.eval_stack.push_managed(addr);
    } else {
        // TODO: what if ele_ty is a reference type?
        unimplemented!();
    }
}

pub fn exec_ldelem_i32(cur_ar: &mut ActivationRecord) {
    let idx = to_arr_size(&cur_ar.eval_stack.pop(None));
    let arr = unsafe { cur_ar.eval_stack.pop(None).expect_ref() };
    let addr = Heap::get_arr_offset(arr, I4_SIZE, idx as usize);
    cur_ar.eval_stack.push_i32(unsafe { *(addr as *const i32) });
}

pub fn exec_stelem_i32(cur_ar: &mut ActivationRecord) {
    let val = cur_ar.eval_stack.pop(None);
    val.expect(SlotTag::I32);
    let idx = to_arr_size(&cur_ar.eval_stack.pop(None));
    let addr = unsafe { cur_ar.eval_stack.pop(None).expect_ref() };
    let addr = Heap::get_arr_offset(addr, I4_SIZE, idx as usize);
    unsafe {
        *(addr as *mut i32) = val.data.i32_;
    }
}

pub fn exec_ldelem_ref(cur_ar: &mut ActivationRecord) {
    let idx = to_arr_size(&cur_ar.eval_stack.pop(None));
    let arr = unsafe { cur_ar.eval_stack.pop(None).expect_ref() };
    let addr = Heap::get_arr_offset(arr, REF_SIZE, idx as usize);
    cur_ar
        .eval_stack
        .push_ptr(unsafe { *(addr as *const *mut u8) });
}

pub fn exec_stelem_ref(cur_ar: &mut ActivationRecord) {
    let val = unsafe { cur_ar.eval_stack.pop(None).expect_ref() };
    let idx = to_arr_size(&cur_ar.eval_stack.pop(None));
    let addr = unsafe { cur_ar.eval_stack.pop(None).expect_ref() };
    let addr = Heap::get_arr_offset(addr, REF_SIZE, idx as usize);
    unsafe {
        *(addr as *mut *mut u8) = val;
    }
}
