use xir::attrib::*;

use super::super::gen::{fn_descriptor, RValType};

pub struct Method {
    pub ret_ty: RValType,
    /// self is not included
    pub ps_ty: Vec<RValType>,
    pub flag: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    /// index into methoddef tbl
    pub idx: u32,
}

impl Method {
    pub fn descriptor(&self) -> String {
        fn_descriptor(&self.ret_ty, &self.ps_ty)
    }
}
