use xir::flag::MethodFlag;

use super::{VMModule, VMType};

pub struct VMMethod {
    pub ctx: VMMethodCtx,

    pub name: u32,

    pub flag: MethodFlag,
    pub ps_ty: Vec<VMType>,
    pub ret_ty: VMType,

    /// if this is a virtual method, offset is the index in vtbl
    pub offset: u32,
    pub locals: usize,
    pub insts: Vec<u8>,
}

pub enum VMMethodCtx {
    Mod(*const VMModule),
    Dll(u32),
}

impl VMMethodCtx {
    pub fn expect_mod(&self) -> *const VMModule {
        match self {
            VMMethodCtx::Mod(m) => *m,
            VMMethodCtx::Dll(_) => panic!(),
        }
    }
}
