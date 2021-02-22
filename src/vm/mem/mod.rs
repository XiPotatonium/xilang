mod heap;
mod slot;
mod stack;
mod static_area;

pub use self::heap::Heap;
pub use self::slot::{Slot, SlotData, SlotTag};
pub use self::stack::Stack;
pub use self::static_area::{StaticArea, VTblEntry};

use super::data::VMModule;

use std::collections::HashMap;
use std::mem::{size_of, transmute};
use std::u32;

pub struct SharedMem {
    pub heap: Heap,
    pub static_area: StaticArea,

    pub mods: HashMap<u32, Box<VMModule>>,

    pub str_pool: Vec<String>,
}

/// default to be 1MB
const HEAP_DEFAULT_SIZE: usize = 0x1 << 6;
// default to be 1MB
const STATIC_DEFAULT_SIZE: usize = 0x1 << 6;

impl SharedMem {
    pub fn new() -> SharedMem {
        SharedMem {
            heap: Heap::new(HEAP_DEFAULT_SIZE),
            static_area: StaticArea::new(STATIC_DEFAULT_SIZE),
            mods: HashMap::new(),
            str_pool: Vec::new(),
        }
    }
}

const ADDR_SIZE: usize = size_of::<usize>() * 8;
const MEM_TAG_MASK: usize = 0x03 << (ADDR_SIZE - 2);

pub enum MemTag {
    HeapMem = 0x02 << (ADDR_SIZE - 2),
    StaticMem = 0x03 << (ADDR_SIZE - 2),
}

pub unsafe fn to_relative(ptr: usize) -> (MemTag, usize) {
    let tag = ptr & MEM_TAG_MASK;
    let offset = ptr & !MEM_TAG_MASK;
    let tag = transmute::<usize, MemTag>(tag);
    (tag, offset)
}

pub unsafe fn to_absolute(tag: MemTag, offset: usize) -> usize {
    if (offset & MEM_TAG_MASK) != 0 {
        panic!("Too large offset");
    }
    transmute::<MemTag, usize>(tag) | offset
}
