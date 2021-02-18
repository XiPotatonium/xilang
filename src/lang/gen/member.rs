use crate::ir::flag::Flag;
use crate::ir::ty::{fn_descriptor, RValType};

pub struct Var {
    pub id: String,
    pub flag: Flag,
    pub ty: RValType,
    pub offset: u16,
    pub initialized: bool,
}

impl Var {
    pub fn new(id: &str, flag: Flag, ty: RValType, offset: u16, initialized: bool) -> Var {
        Var {
            id: id.to_owned(),
            flag,
            ty,
            offset,
            initialized,
        }
    }
}

pub struct Field {
    pub id: String,
    pub flag: Flag,
    pub ty: RValType,
}

impl Field {
    pub fn new(id: &str, flag: Flag, ty: RValType) -> Field {
        Field {
            id: id.to_owned(),
            flag,
            ty,
        }
    }
}

pub struct Method {
    pub flag: Flag,
    pub ret_ty: RValType,
    /// self is not included
    pub ps_ty: Vec<RValType>,
    /// method idx in class file
    pub method_idx: usize,
}

impl Method {
    pub fn descriptor(&self) -> String {
        fn_descriptor(&self.ret_ty, &self.ps_ty)
    }
}
