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
use std::u32;

pub struct SharedMem {
    pub heap: Heap,
    pub static_area: StaticArea,

    pub mods: HashMap<u32, Box<VMModule>>,

    pub str_pool: Vec<String>,
}

impl SharedMem {
    pub fn new() -> SharedMem {
        SharedMem {
            heap: Heap::new(),
            static_area: StaticArea::new(),
            mods: HashMap::new(),
            str_pool: Vec::new(),
        }
    }
}
