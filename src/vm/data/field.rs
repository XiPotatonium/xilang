use crate::ir::flag::FieldFlag;

use super::VMType;

pub struct VMField {
    pub name: u32,
    pub flag: FieldFlag,
    pub ty: VMType,

    /// for static field, this is address in memory;
    /// for non static field, this is the offset to the start of object
    pub addr: usize,
}
