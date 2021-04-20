use xir::attrib::{FieldAttrib, MethodAttrib, MethodImplAttrib, ParamAttrib};

use super::super::gen::RValType;

pub struct Method {
    pub ret: RValType,
    /// self is not included
    pub ps: Vec<Param>,
    pub attrib: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    /// index into methoddef tbl
    pub idx: u32,
}

pub struct Param {
    pub id: String,
    pub attrib: ParamAttrib,
    pub ty: RValType,
}

pub struct Field {
    pub attrib: FieldAttrib,
    pub ty: RValType,

    /// index into field tbl
    pub idx: u32,
}
