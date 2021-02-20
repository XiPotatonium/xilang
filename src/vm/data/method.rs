use crate::ir::flag::MethodFlag;
use crate::ir::inst::Inst;

pub struct VMMethod {
    pub flag: MethodFlag,
    pub insts: Vec<Inst>,
}
