use crate::ir::flag::FieldFlag;

use super::VMType;

pub struct VMField {
    pub name: u32,
    pub flag: FieldFlag,
    pub ty: VMType,
    ///
    pub offset: u32,
}
