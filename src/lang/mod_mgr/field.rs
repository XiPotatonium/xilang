use super::super::gen::RValType;
use xir::flag::*;

pub struct Field {
    pub id: String,
    pub flag: FieldFlag,
    pub ty: RValType,
    /// field idx in class file
    pub field_idx: u32,
}

impl Field {
    pub fn new(id: &str, flag: FieldFlag, ty: RValType, idx: u32) -> Field {
        Field {
            id: id.to_owned(),
            flag,
            ty,
            field_idx: idx,
        }
    }
}
