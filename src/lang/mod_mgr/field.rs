use super::super::gen::RValType;
use xir::attrib::*;

pub struct Field {
    pub id: String,
    pub attrib: FieldAttrib,
    pub ty: RValType,
    /// index into field tbl
    pub idx: u32,
}

impl Field {
    pub fn new(id: &str, attrib: FieldAttrib, ty: RValType, idx: u32) -> Field {
        Field {
            id: id.to_owned(),
            attrib,
            ty,
            idx,
        }
    }
}
