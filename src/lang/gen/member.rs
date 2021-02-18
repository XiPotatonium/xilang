use crate::ir::flag::*;
use crate::ir::ty::{fn_descriptor, RValType};

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

pub struct Method {
    pub flag: MethodFlag,
    pub ret_ty: RValType,
    pub ps_flag: Vec<ParamFlag>,
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
