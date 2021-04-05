use xir::attrib::{MethodAttrib, MethodImplAttrib, PInvokeAttrib};

use super::{VMModule, VMType};

pub struct VMMethod {
    pub ctx: *const VMModule,

    pub name: u32,

    pub flag: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    pub ps_ty: Vec<VMType>,
    pub ret_ty: VMType,

    pub method_impl: VMMethodImpl,
}

pub enum VMMethodImpl {
    IL(VMMethodILImpl),
    Native(VMMethodNativeImpl),
}

impl VMMethodImpl {
    pub fn expect_il(&self) -> &VMMethodILImpl {
        match self {
            VMMethodImpl::IL(method_impl) => method_impl,
            VMMethodImpl::Native(_) => panic!(),
        }
    }

    pub fn expect_il_mut(&mut self) -> &mut VMMethodILImpl {
        match self {
            VMMethodImpl::IL(method_impl) => method_impl,
            VMMethodImpl::Native(_) => panic!(),
        }
    }
}

pub struct VMMethodILImpl {
    pub offset: u32,
    pub locals: usize,
    pub insts: Vec<u8>,
}

pub struct VMMethodNativeImpl {
    pub name: u32,
    pub flag: PInvokeAttrib,
}
