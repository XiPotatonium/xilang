use crate::ir::flag::*;
use crate::ir::ty::IrValType;

pub struct Field {
    pub id: String,
    pub flag: FieldFlag,
    pub ty: IrValType,
}

impl Field {
    pub fn new(id: &str, flag: FieldFlag, ty: IrValType) -> Field {
        Field {
            id: id.to_owned(),
            flag,
            ty,
        }
    }
}
