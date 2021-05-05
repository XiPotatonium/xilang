use super::data::{Module, Type};
use super::heap::Heap;

use std::collections::HashMap;
use std::ptr;

pub struct SharedMem {
    heap: Heap,

    /// name -> module
    pub mods: HashMap<usize, Box<Module>>,

    pub str_pool: Vec<String>,

    /// index of "std"
    pub std_str_idx: usize,
    /// index of ""
    pub empty_str_idx: usize,
    pub str_class: *const Type,
}

/// default to be 1MB
const HEAP_DEFAULT_SIZE: usize = 0x1 << 20;

impl SharedMem {
    pub fn new() -> SharedMem {
        SharedMem {
            heap: Heap::new(HEAP_DEFAULT_SIZE),
            mods: HashMap::new(),
            str_pool: Vec::new(),

            std_str_idx: 0,
            empty_str_idx: 0,
            str_class: ptr::null(),
        }
    }
}

impl SharedMem {
    pub unsafe fn new_obj(&mut self, class: *const Type) -> *mut u8 {
        self.heap.new_obj(class)
    }

    pub unsafe fn new_str_from_str(&mut self, s: usize) -> *mut u8 {
        self.heap
            .new_str_from_str(self.str_class, &self.str_pool[s])
    }
}
