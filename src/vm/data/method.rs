use xir::attrib::{MethodAttrib, MethodImplAttrib, PInvokeAttrib};

use super::{VMBuiltinType, VMModule};

pub struct VMMethod {
    pub ctx: *const VMModule,

    pub name: u32,

    pub flag: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    pub ps_ty: Vec<VMBuiltinType>,
    pub ret_ty: VMBuiltinType,

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
}

pub struct VMMethodILImpl {
    pub offset: u32,
    pub locals: usize,
    pub insts: Vec<u8>,
}

pub struct VMMethodNativeImpl {
    // index of modref (dll)
    pub scope: usize,
    pub name: u32,
    pub flag: PInvokeAttrib,
}
