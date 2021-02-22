use std::mem::size_of;

pub struct Heap {
    next_obj_offset: usize,
    data: Vec<u8>,
}

impl Heap {
    pub fn new(size: usize) -> Heap {
        Heap {
            data: vec![0; size],
            next_obj_offset: 0,
        }
    }

    /// New obj
    ///
    /// [header: usize] [vtbl: *VTbl] [content...]
    pub fn new_obj(&mut self, n: usize) -> usize {
        let ret = self.next_obj_offset + size_of::<usize>() * 2;
        self.next_obj_offset = ret + n;
        ret
    }

    pub fn access<T>(&self, offset: usize) -> *const T {
        &self.data[offset] as *const u8 as *const T
    }

    pub fn access_mut<T>(&mut self, offset: usize) -> *mut T {
        &mut self.data[offset] as *mut u8 as *mut T
    }
}
