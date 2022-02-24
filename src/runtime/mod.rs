use crate::lang::build::FileLoader;

mod interp;
mod mem;

use interp::interp;
use mem::{ActivationRecord, Heap};

const HEAP_INIT_SIZE: usize = 1024; // 1024bytes

pub fn exec(loader: &FileLoader) {
    println!("This is xi runtime");
    if let Some(entry) = loader.crates[0].funcs.get("main") {
        let mut heap = Heap::new(HEAP_INIT_SIZE);
        let mut stack: Vec<ActivationRecord> = Vec::new();
        interp(loader, &mut heap, &mut stack);
    } else {
        panic!("entry not found");
    }
}
