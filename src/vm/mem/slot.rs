use std::mem::transmute;

use super::{to_absolute, to_relative, MemTag};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Slot {
    pub tag: SlotTag,
    pub data: SlotData,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub enum SlotTag {
    I32,
    I64,
    INative,
    Ref,
    F,
    Uninit,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union SlotData {
    pub i32_: i32,
    pub i64_: i64,
    pub inative_: isize,
    pub unative_: usize,
}

impl Default for Slot {
    fn default() -> Self {
        Slot {
            tag: SlotTag::Uninit,
            data: SlotData { i32_: 0 },
        }
    }
}

impl Slot {
    pub fn null() -> Self {
        Slot {
            tag: SlotTag::Ref,
            data: SlotData { unative_: 0 },
        }
    }

    pub unsafe fn new_ref(tag: MemTag, offset: usize) -> Slot {
        Slot {
            tag: SlotTag::Ref,
            data: SlotData {
                unative_: to_absolute(tag, offset),
            },
        }
    }

    pub fn as_u32(&self) -> u32 {
        unsafe { transmute::<i32, u32>(self.data.i32_) }
    }

    /// interpret as ptr and map into relative address
    ///
    ///
    pub unsafe fn as_addr(&self) -> (MemTag, usize) {
        if let SlotTag::Ref = self.tag {
            to_relative(self.data.unative_)
        } else {
            panic!("Slot is not ptr");
        }
    }
}
