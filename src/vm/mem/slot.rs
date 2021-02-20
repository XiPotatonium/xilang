#[derive(Clone)]
pub struct Slot {
    tag: SlotTag,
    data: SlotData,
}

#[derive(Clone)]
pub enum SlotTag {
    I32,
    F64,
    Ref,
    Unint,
}

#[derive(Copy, Clone)]
pub union SlotData {
    i32_: i32,
    f64_: f64,
    ref_: u32,
}

impl Default for Slot {
    fn default() -> Self {
        Slot {
            tag: SlotTag::Unint,
            data: SlotData { i32_: 0 },
        }
    }
}
