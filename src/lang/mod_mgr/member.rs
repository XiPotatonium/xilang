use std::collections::HashMap;

use xir::attrib::{FieldAttrib, MethodAttrib, MethodImplAttrib, ParamAttrib};

use super::super::gen::RValType;

pub struct Method {
    pub ret: RValType,
    /// self is not included
    pub ps: Vec<Param>,
    /// map from ps name to ps idx
    pub ps_map: HashMap<String, usize>,
    pub flag: MethodAttrib,
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
