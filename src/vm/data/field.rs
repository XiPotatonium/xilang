use xir::attrib::FieldAttrib;

use super::BuiltinType;

pub struct Field {
    pub name: u32,
    pub attrib: FieldAttrib,
    pub ty: BuiltinType,

    /// for static field, this is address in memory;
    /// for non static field, this is the offset to the start of object
    pub addr: usize,
}
