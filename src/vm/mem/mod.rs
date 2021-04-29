mod heap;
pub mod slot;
mod stack;
mod static_area;

pub use self::heap::Heap;
pub use self::slot::{Slot, SlotData, SlotTag};
pub use self::stack::Stack;
pub use self::static_area::{StaticArea, VTblEntry};

use super::data::Module;

use std::collections::HashMap;
use std::convert::{From, TryFrom};

pub struct SharedMem {
    pub heap: Heap,
    pub static_area: StaticArea,

    /// name -> module
    pub mods: HashMap<usize, Box<Module>>,

    pub str_pool: Vec<String>,
}

/// default to be 1MB
const HEAP_DEFAULT_SIZE: usize = 0x1 << 20;
// default to be 1MB
const STATIC_DEFAULT_SIZE: usize = 0x1 << 20;

impl SharedMem {
    pub fn new() -> SharedMem {
        SharedMem {
            heap: Heap::new(HEAP_DEFAULT_SIZE),
            static_area: StaticArea::new(STATIC_DEFAULT_SIZE),
            mods: HashMap::new(),
            str_pool: Vec::new(),
        }
    }

    pub fn get_str(&self, i: usize) -> &str {
        &self.str_pool[i]
    }
}

// 2 bits mem tag
const MEM_TAG_MASK_SIZE: usize = 2;
const MEM_TAG_MASK: usize = 0x3;
const MEM_TAG_HEAP: usize = 0x02;
const MEM_TAG_STATIC: usize = 0x03;

pub enum MemTag {
    HeapMem,
    StaticMem,
}

impl From<MemTag> for usize {
    fn from(value: MemTag) -> Self {
        match value {
            MemTag::HeapMem => MEM_TAG_HEAP,
            MemTag::StaticMem => MEM_TAG_STATIC,
        }
    }
}

impl TryFrom<usize> for MemTag {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            MEM_TAG_HEAP => Ok(MemTag::HeapMem),
            MEM_TAG_STATIC => Ok(MemTag::StaticMem),
            _ => unreachable!(),
        }
    }
}

pub fn to_relative(ptr: usize) -> (MemTag, usize) {
    let offset = ptr >> MEM_TAG_MASK_SIZE;
    let tag = MemTag::try_from(ptr & MEM_TAG_MASK).unwrap();
    (tag, offset)
}

pub fn to_absolute(tag: MemTag, offset: usize) -> usize {
    // TODO: check overflow
    offset << MEM_TAG_MASK_SIZE | usize::from(tag)
}

pub fn addr_addi(ptr: usize, offset: isize) -> usize {
    // TODO: check overflow and underflow
    if offset > 0 {
        ptr + ((offset as usize) << MEM_TAG_MASK_SIZE)
    } else {
        ptr + ((-offset as usize) << MEM_TAG_MASK_SIZE)
    }
}

pub fn addr_addu(ptr: usize, offset: usize) -> usize {
    // TODO: check overflow
    ptr + (offset << MEM_TAG_MASK_SIZE)
}
