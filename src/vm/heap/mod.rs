mod obj;

use std::mem::size_of;

use obj::{ArrHeader, ObjHeader, StrHeader};

use super::data::Type;

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

    pub fn get_vtbl_ptr(instance_ptr: *mut u8) -> *const Type {
        unsafe {
            *(instance_ptr.wrapping_sub(size_of::<*const *const Type>()) as *const *const Type)
        }
    }

    /// New obj
    ///
    /// [ObjHeader] [content...]
    pub unsafe fn new_obj(&mut self, class: *const Type) -> *mut u8 {
        let class = class.as_ref().unwrap();

        let offset_after_alloc =
            class.basic_instance_size + size_of::<ObjHeader>() + self.next_obj_offset;
        if offset_after_alloc >= self.data.len() {
            // GC
            unimplemented!("GC");
        }

        (&mut self.data[self.next_obj_offset] as *mut u8 as *mut ObjHeader)
            .as_mut()
            .unwrap()
            .init(class);
        let ret = &mut self.data[self.next_obj_offset + size_of::<ObjHeader>()] as *mut u8;
        self.next_obj_offset = offset_after_alloc;
        ret
    }

    /// [StrHeader] [elementes...]
    ///
    /// str_class must point to std::String
    pub unsafe fn new_str_from_str(&mut self, str_class: *const Type, s: &str) -> *mut u8 {
        let char_count = s.chars().count();
        let offset_after_alloc =
            char_count * size_of::<char>() + size_of::<StrHeader>() + self.next_obj_offset;
        if offset_after_alloc >= self.data.len() {
            // GC
            unimplemented!("GC");
        }

        (&mut self.data[self.next_obj_offset] as *mut u8 as *mut StrHeader)
            .as_mut()
            .unwrap()
            .init(str_class, char_count);
        let ret = &mut self.data[self.next_obj_offset + size_of::<ObjHeader>()] as *mut u8;
        self.next_obj_offset = offset_after_alloc;
        ret
    }
}
