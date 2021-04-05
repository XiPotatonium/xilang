use xir::attrib::FieldAttrib;

use super::VMType;

pub struct VMField {
    pub name: u32,
    pub attrib: FieldAttrib,
    pub ty: VMType,

    /// for static field, this is address in memory;
    /// for non static field, this is the offset to the start of object
    pub addr: usize,
}
