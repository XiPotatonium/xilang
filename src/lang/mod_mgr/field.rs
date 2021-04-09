use super::super::gen::RValType;
use xir::attrib::*;

pub struct Field {
    pub flag: FieldAttrib,
    pub ty: RValType,
    /// index into field tbl
    pub idx: u32,
}

impl Field {
    pub fn new(attrib: FieldAttrib, ty: RValType, idx: u32) -> Field {
        Field {
            flag: attrib,
            ty,
            idx,
        }
    }
}
