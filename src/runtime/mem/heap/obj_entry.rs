use std::mem::size_of;

use std::marker::PhantomData;

use crate::lang::sym::Class;

#[repr(C)]
pub struct ObjHeader {
    pub flag: usize,
    pub p_method_tbl: *const Class,
}

impl ObjHeader {
    pub fn init(&mut self, class: *const Class) {
        self.p_method_tbl = class;
    }
}

#[repr(C)]
pub struct ArrHeader {
    pub obj_header: ObjHeader,
    pub len: usize,
}

impl ArrHeader {
    pub fn init(&mut self, class: *const Class, len: usize) {
        self.obj_header.init(class);
        self.len = len;
    }
}

#[repr(C)]
pub struct StrHeader {
    pub obj_header: ObjHeader,
    pub len: usize,
}

impl StrHeader {
    pub fn init(&mut self, class: *const Class, len: usize) {
        self.obj_header.init(class);
        self.len = len;
    }
}

pub struct StrCharsIter {
    total_chars_len: usize,
    current_char_idx: usize,
    current_ptr: *const char,
}

impl StrCharsIter {
    pub fn new(str_ptr: *mut u8) -> StrCharsIter {
        let str_len = unsafe { *(str_ptr as *const usize) };
        StrCharsIter {
            total_chars_len: str_len,
            current_char_idx: 0,
            current_ptr: str_ptr.wrapping_add(size_of::<usize>()) as *const char,
        }
    }

    pub fn total_len(&self) -> usize {
        self.total_chars_len
    }
}

impl Iterator for StrCharsIter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_char_idx < self.total_chars_len {
            let ret = Some(unsafe { *self.current_ptr });

            self.current_char_idx += 1;
            self.current_ptr = self.current_ptr.wrapping_add(1);

            ret
        } else {
            None
        }
    }
}

pub struct StrCharsIterMut<'a> {
    total_chars_len: usize,
    current_char_idx: usize,
    current_ptr: *mut char,
    _marker: PhantomData<&'a mut char>,
}

impl<'a> StrCharsIterMut<'a> {
    pub fn new(str_ptr: *mut u8) -> StrCharsIterMut<'a> {
        let str_len = unsafe { *(str_ptr as *const usize) };
        StrCharsIterMut {
            total_chars_len: str_len,
            current_char_idx: 0,
            current_ptr: str_ptr.wrapping_add(size_of::<usize>()) as *mut char,
            _marker: PhantomData,
        }
    }

    pub fn total_len(&self) -> usize {
        self.total_chars_len
    }
}

impl<'a> Iterator for StrCharsIterMut<'a> {
    type Item = &'a mut char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_char_idx < self.total_chars_len {
            let ret = Some(unsafe { self.current_ptr.as_mut().unwrap() });

            self.current_char_idx += 1;
            self.current_ptr = self.current_ptr.wrapping_add(1);

            ret
        } else {
            None
        }
    }
}
