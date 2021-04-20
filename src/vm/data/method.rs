use xir::attrib::{MethodAttrib, MethodImplAttrib, PInvokeAttrib, ParamAttrib};

use super::{BuiltinType, Module, Type};

pub struct Param {
    pub name: u32,
    pub attrib: ParamAttrib,
    pub ty: BuiltinType,
}

pub struct Method {
    pub ctx: *const Module,
    pub parent_class: Option<*const Type>,

    pub name: u32,

    pub flag: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    pub ps: Vec<Param>,
    pub ret: Param,

    pub method_impl: MethodImpl,
}

pub enum MethodImpl {
    IL(MethodILImpl),
    Native(MethodNativeImpl),
}

impl MethodImpl {
    pub fn expect_il(&self) -> &MethodILImpl {
        match self {
            MethodImpl::IL(method_impl) => method_impl,
            MethodImpl::Native(_) => panic!(),
        }
    }
}

pub struct MethodILImpl {
    pub offset: u32,
    pub locals: Vec<BuiltinType>,
    pub insts: Vec<u8>,
}

pub struct MethodNativeImpl {
    // index of modref (dll)
    pub scope: usize,
    pub name: u32,
    pub flag: PInvokeAttrib,
}
