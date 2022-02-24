mod heap;
mod stack;

use std::mem;

use crate::lang::sym::{Class, RValType};

pub use self::heap::Heap;
pub use self::stack::ActivationRecord;
use self::stack::{EvalStack, Slot, SlotTag};

// load from addr into stack
unsafe fn load(ty: &RValType, addr: *const u8, stack: &mut EvalStack) {
    match ty {
        RValType::None => panic!("Cannot store none"),
        RValType::U8 | RValType::Bool => {
            stack.push_i32(mem::transmute::<u32, i32>(*addr as u32));
        }
        RValType::Char => unimplemented!(),
        RValType::I32 => stack.push_i32(*(addr as *const i32)),
        RValType::USize => unimplemented!(),
        RValType::ISize => unimplemented!(),
        RValType::F64 => unimplemented!(),
        RValType::ClassRef(_) => stack.push_ref(*(addr as *const *mut u8)),
        RValType::Array(_) => todo!(),
        RValType::UnInit => unreachable!(),
    }
}

// store slot into addr
unsafe fn store_slot(ty: &RValType, addr: *mut u8, slot: Slot) {
    match ty {
        RValType::None => panic!("Cannot store none"),
        RValType::Bool | RValType::U8 => {
            slot.expect(SlotTag::I32);
            *addr = mem::transmute::<i32, u32>(slot.data.i32_) as u8;
        }
        RValType::Char => unimplemented!(),
        RValType::I32 => {
            slot.expect(SlotTag::I32);
            *(addr as *mut i32) = slot.data.i32_;
        }
        RValType::USize => unimplemented!(),
        RValType::ISize => unimplemented!(),
        RValType::F64 => unimplemented!(),
        RValType::ClassRef(_) => {
            *(addr as *mut *mut u8) = slot.data.ptr_;
        }
        RValType::Array(_) => todo!(),
        RValType::UnInit => unreachable!(),
    }
}

pub fn heap_size(ty: &RValType) -> usize {
    match ty {
        RValType::None => 0,
        RValType::Bool => mem::size_of::<bool>(),
        RValType::U8 => mem::size_of::<u8>(),
        RValType::Char => mem::size_of::<char>(),
        RValType::I32 => mem::size_of::<i32>(),
        RValType::F64 => mem::size_of::<f64>(),
        RValType::ISize => mem::size_of::<isize>(),
        RValType::USize => mem::size_of::<usize>(),
        RValType::ClassRef(_) | RValType::Array(_) => mem::size_of::<*const u8>(),
        RValType::UnInit => unreachable!(),
    }
}

/// object head is not considered
pub fn object_size(ty: &Class) -> usize {
    let mut sze = 0;
    for f in ty.fields.values() {
        sze += heap_size(&f.ty);
    }
    sze
}
