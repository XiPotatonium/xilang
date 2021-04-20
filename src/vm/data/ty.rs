use xir::attrib::TypeAttrib;

use super::{Field, Method};

pub struct Type {
    pub name: u32,
    pub attrib: TypeAttrib,

    // ownership of methods and fields is at parent module
    pub methods: Vec<*mut Method>,
    pub fields: Vec<*mut Field>,

    pub vtbl_addr: usize,
    pub obj_size: usize,
}
