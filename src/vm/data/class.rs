use crate::ir::flag::TypeFlag;

use super::{VMField, VMMethod};

pub struct VMClass {
    pub name: u32,
    pub flag: TypeFlag,

    // ownership of methods and fields is at parent module
    pub methods: Vec<*const VMMethod>,
    pub fields: Vec<*const VMField>,

    pub vtbl_addr: usize,
    pub obj_size: usize,
}
