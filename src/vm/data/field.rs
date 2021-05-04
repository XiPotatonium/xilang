use xir::attrib::FieldAttrib;

use super::BuiltinType;

pub struct Field {
    pub name: usize,
    pub attrib: FieldAttrib,
    pub ty: BuiltinType,

    /// for static field, this is offset in Type.static_fields
    /// for instance field, this is the offset to the start of object
    pub offset: usize,
    // &parent.static_fields[self.offset]
    pub addr: *mut u8,
}
