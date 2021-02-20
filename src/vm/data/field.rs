use crate::ir::flag::FieldFlag;

pub struct VMField {
    pub flag: FieldFlag,
    pub offset: u32,
}
