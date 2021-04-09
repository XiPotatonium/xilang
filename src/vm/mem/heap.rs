use std::mem::size_of;

use super::super::data::VMType;

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
    pub unsafe fn new_obj(&mut self, class: *const VMType) -> usize {
        let class = class.as_ref().unwrap();

        if class.obj_size + size_of::<usize>() * 2 + self.next_obj_offset >= self.data.len() {
            // GC
            unimplemented!("GC");
        }

        let pvtbl =
            &mut self.data[self.next_obj_offset + size_of::<usize>()] as *mut u8 as *mut usize;
        *pvtbl = class.vtbl_addr;
        let ret = self.next_obj_offset + size_of::<usize>() * 2;
        self.next_obj_offset = ret + class.obj_size;
        ret
    }

    pub fn access<T>(&self, offset: usize) -> *const T {
        &self.data[offset] as *const u8 as *const T
    }

    pub fn access_mut<T>(&mut self, offset: usize) -> *mut T {
        &mut self.data[offset] as *mut u8 as *mut T
    }
}
