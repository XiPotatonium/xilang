use super::data::Module;
use super::heap::Heap;

use std::collections::HashMap;

pub struct SharedMem {
    pub heap: Heap,

    /// name -> module
    pub mods: HashMap<usize, Box<Module>>,

    pub str_pool: Vec<String>,
}

/// default to be 1MB
const HEAP_DEFAULT_SIZE: usize = 0x1 << 20;

impl SharedMem {
    pub fn new() -> SharedMem {
        SharedMem {
            heap: Heap::new(HEAP_DEFAULT_SIZE),
            mods: HashMap::new(),
            str_pool: Vec::new(),
        }
    }

    pub fn get_str(&self, i: usize) -> &str {
        &self.str_pool[i]
    }
}
