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
use std::mem::size_of;
use std::u32;

pub struct SharedMem {
    pub heap: Heap,
    pub static_area: StaticArea,

    pub mods: HashMap<u32, Box<VMModule>>,

    pub str_pool: Vec<String>,
}

/// default to be 1MB
const HEAP_DEFAULT_SIZE: usize = 0x1 << 6;

impl SharedMem {
    pub fn new() -> SharedMem {
        SharedMem {
            heap: Heap::new(HEAP_DEFAULT_SIZE),
            static_area: StaticArea::new(),
            mods: HashMap::new(),
            str_pool: Vec::new(),
        }
    }
}

const ADDR_SIZE: usize = size_of::<usize>();
