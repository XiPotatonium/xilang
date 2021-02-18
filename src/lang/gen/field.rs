use crate::ir::flag::*;
use crate::ir::ty::RValType;

pub struct Field {
    pub id: String,
    pub flag: FieldFlag,
    pub ty: RValType,
}

impl Field {
    pub fn new(id: &str, flag: FieldFlag, ty: RValType) -> Field {
        Field {
            id: id.to_owned(),
            flag,
            ty,
        }
    }
}
