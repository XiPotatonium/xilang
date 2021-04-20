use xir::attrib::{MethodAttrib, MethodImplAttrib, PInvokeAttrib, ParamAttrib};

use super::{VMBuiltinType, VMModule, VMType};

pub struct VMParam {
    pub name: u32,
    pub attrib: ParamAttrib,
    pub ty: VMBuiltinType,
}

pub struct VMMethod {
    pub ctx: *const VMModule,
    pub parent_class: Option<*const VMType>,

    pub name: u32,

    pub flag: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    pub ps: Vec<VMParam>,
    pub ret: VMParam,

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
    pub locals: Vec<VMBuiltinType>,
    pub insts: Vec<u8>,
}

pub struct VMMethodNativeImpl {
    // index of modref (dll)
    pub scope: usize,
    pub name: u32,
    pub flag: PInvokeAttrib,
}
