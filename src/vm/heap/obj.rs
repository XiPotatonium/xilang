use super::super::data::Type;

#[repr(C)]
pub struct ObjHeader {
    pub flag: usize,
    pub p_method_tbl: *const Type,
}

impl ObjHeader {
    pub fn init(&mut self, class: *const Type) {
        self.p_method_tbl = class;
    }
}

#[repr(C)]
pub struct ArrHeader {
    pub obj_header: ObjHeader,
    pub len: usize,
}

#[repr(C)]
pub struct StrHeader {
    pub obj_header: ObjHeader,
    pub len: usize,
}

impl StrHeader {
    pub fn init(&mut self, class: *const Type, len: usize) {
        self.obj_header.init(class);
        self.len = len;
    }
}
