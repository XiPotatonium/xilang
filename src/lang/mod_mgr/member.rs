use xir::attrib::{FieldAttrib, MethodAttrib, MethodImplAttrib, ParamAttrib};

use super::super::gen::RValType;
use super::Class;

use std::fmt;

pub struct Method {
    pub parent: *const Class,

    pub name: String,

    pub ret: RValType,
    /// self is not included
    pub ps: Vec<Param>,
    pub attrib: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    /// index into methoddef tbl
    pub idx: u32,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

pub struct Param {
    pub id: String,
    pub attrib: ParamAttrib,
    pub ty: RValType,
}

pub struct Field {
    pub parent: *const Class,

    pub name: String,

    pub attrib: FieldAttrib,
    pub ty: RValType,

    /// index into field tbl
    pub idx: u32,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
