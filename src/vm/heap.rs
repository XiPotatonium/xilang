use std::mem::size_of;

use super::data::Type;

type VTblPtr = *const Type;

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

    pub fn get_vtbl_ptr(instance_ptr: *mut u8) -> *mut Type {
        unsafe { *(instance_ptr.wrapping_sub(size_of::<VTblPtr>()) as *const *mut Type) }
    }

    /// New obj
    ///
    /// [header: usize] [vtblptr: *const Type] [content...]
    pub unsafe fn new_obj(&mut self, class: *const Type) -> *mut u8 {
        let class = class.as_ref().unwrap();

        let offset_after_alloc = class.basic_instance_size
            + size_of::<usize>()
            + size_of::<VTblPtr>()
            + self.next_obj_offset;
        if offset_after_alloc >= self.data.len() {
            // GC
            unimplemented!("GC");
        }

        let pvtbl =
            &mut self.data[self.next_obj_offset + size_of::<usize>()] as *mut u8 as *mut VTblPtr;
        *pvtbl = class as *const Type;
        let ret = &mut self.data[self.next_obj_offset + size_of::<usize>() + size_of::<VTblPtr>()]
            as *mut u8;
        self.next_obj_offset = offset_after_alloc;
        ret
    }
}
