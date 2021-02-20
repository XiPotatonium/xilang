mod heap;
mod slot;
mod stack;
mod static_area;

pub use self::heap::Heap;
pub use self::slot::{Slot, SlotData, SlotTag};
pub use self::stack::Stack;
pub use self::static_area::StaticArea;

use super::data::VMModule;

use std::collections::HashMap;

pub struct SharedMem {
    pub heap: Heap,
    pub static_area: StaticArea,

    pub mods: HashMap<u32, VMModule>,

    pub str_pool: Vec<String>,
}

impl SharedMem {
    pub fn new() -> SharedMem {
        unimplemented!();
    }

    pub fn add_const_str(&mut self, s: String) -> u32 {
        unimplemented!();
    }
}
