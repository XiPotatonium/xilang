use super::super::gen::RValType;
use xir::flag::*;

pub struct Field {
    pub id: String,
    pub flag: FieldFlag,
    pub ty: RValType,
    /// index into field tbl
    pub idx: u32,
}

impl Field {
    pub fn new(id: &str, flag: FieldFlag, ty: RValType, idx: u32) -> Field {
        Field {
            id: id.to_owned(),
            flag,
            ty,
            idx,
        }
    }
}
