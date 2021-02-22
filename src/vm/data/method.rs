use crate::ir::flag::MethodFlag;
use crate::ir::inst::Inst;

use super::VMType;

pub struct VMMethod {
    pub name: u32,

    pub flag: MethodFlag,
    pub ps_ty: Vec<VMType>,
    pub ret_ty: VMType,

    /// if this is a virtual method, offset is the index in vtbl
    pub offset: u32,
    pub locals: usize,
    pub insts: Vec<Inst>,
}
