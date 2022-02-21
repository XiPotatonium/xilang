mod obj_entry;

use std::mem::size_of;

use crate::lang::sym::Class;
use obj_entry::{ArrHeader, ObjHeader, StrCharsIter, StrHeader, StrCharsIterMut};


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

    pub fn get_arr_offset(self_ptr: *mut u8, ele_size: usize, idx: usize) -> *mut u8 {
        // TODO use array type to determine element size rather than
        let arr_entry_ptr = self_ptr.wrapping_sub(size_of::<ObjHeader>());
        let len = unsafe { (arr_entry_ptr as *const ArrHeader).as_ref().unwrap().len };
        if len <= idx {
            panic!("Accessing array of length {} with index {}", len, idx);
        }
        arr_entry_ptr.wrapping_add(ele_size * idx + size_of::<ArrHeader>())
    }

    pub fn get_arr_len(self_ptr: *mut u8) -> usize {
        unsafe { Self::get_entry::<ArrHeader>(self_ptr).as_ref().unwrap().len }
    }

    pub fn get_vtbl_ptr(self_ptr: *mut u8) -> *const Class {
        unsafe {
            Self::get_entry::<ObjHeader>(self_ptr)
                .as_ref()
                .unwrap()
                .p_method_tbl
        }
    }

    /// T can be ObjHeader/ArrHeader/StrHeader
    pub fn get_entry<T>(self_ptr: *mut u8) -> *const T {
        self_ptr.wrapping_sub(size_of::<ObjHeader>()) as *const T
    }

    pub fn get_chars(self_ptr: *mut u8) -> StrCharsIter {
        StrCharsIter::new(self_ptr)
    }

    /// New obj
    ///
    /// [ObjHeader] [content...]
    pub unsafe fn new_obj(&mut self, class: &Class) -> *mut u8 {
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

    /// [StrHeader] [chars...]
    ///
    /// str_class must point to std::String
    pub unsafe fn new_str_from_str(&mut self, str_class: *const Class, s: &str) -> *mut u8 {
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
        let chars = StrCharsIterMut::new(ret);
        for (ch_target, ch_src) in chars.zip(s.chars()) {
            *ch_target = ch_src;
        }
        ret
    }

    /// [ArrHeader] [elements]
    pub unsafe fn new_arr(
        &mut self,
        arr_class: *const Class,
        ele_size: usize,
        len: usize,
    ) -> *mut u8 {
        // TODO: ele_ty can be contained in arr_class
        // TODO: check value type
        let offset_after_alloc = len * ele_size + size_of::<ArrHeader>() + self.next_obj_offset;
        if offset_after_alloc >= self.data.len() {
            // GC
            unimplemented!("GC");
        }

        (&mut self.data[self.next_obj_offset] as *mut u8 as *mut ArrHeader)
            .as_mut()
            .unwrap()
            .init(arr_class, len);
        let ret = &mut self.data[self.next_obj_offset + size_of::<ObjHeader>()] as *mut u8;
        self.next_obj_offset = offset_after_alloc;
        ret
    }
}