use xir::attrib::TypeAttrib;

use super::{VMField, VMMethod};

pub struct VMType {
    pub name: u32,
    pub attrib: TypeAttrib,

    // ownership of methods and fields is at parent module
    pub methods: Vec<*mut VMMethod>,
    pub fields: Vec<*mut VMField>,

    pub vtbl_addr: usize,
    pub obj_size: usize,
}
