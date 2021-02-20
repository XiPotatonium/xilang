use crate::ir::flag::*;
use crate::ir::ty::{fn_descriptor, IrValType};

pub struct Method {
    pub flag: MethodFlag,
    pub ret_ty: IrValType,
    pub ps_flag: Vec<ParamFlag>,
    /// self is not included
    pub ps_ty: Vec<IrValType>,
    /// method idx in class file
    pub method_idx: u32,
}

impl Method {
    pub fn descriptor(&self) -> String {
        fn_descriptor(&self.ret_ty, &self.ps_ty)
    }
}
