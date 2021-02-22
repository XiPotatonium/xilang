use std::mem::transmute;

#[derive(Clone, Copy)]
pub struct Slot {
    pub tag: SlotTag,
    pub data: SlotData,
}

#[derive(Clone, Copy)]
pub enum SlotTag {
    I32,
    I64,
    F64,
    Ref,
    Uninit,
}

#[derive(Copy, Clone)]
pub union SlotData {
    pub i32_: i32,
    pub i64_: i64,
    pub f64_: f64,
    pub ref_: usize,
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
            data: SlotData { ref_: 0 },
        }
    }

    pub fn as_u32(&self) -> u32 {
        unsafe { transmute::<i32, u32>(self.data.i32_) }
    }
}
