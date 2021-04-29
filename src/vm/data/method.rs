use xir::attrib::{MethodAttrib, MethodImplAttrib, PInvokeAttrib, ParamAttrib};

use super::{BuiltinType, Module, Type};

pub struct Param {
    pub name: usize,
    pub attrib: ParamAttrib,
    pub ty: BuiltinType,
}

pub struct Method {
    /// module where method is declared
    pub ctx: *const Module,
    /// None if parent is ctx
    pub parent: Option<*const Type>,

    pub name: usize,

    pub attrib: MethodAttrib,
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
    pub name: usize,
    pub flag: PInvokeAttrib,
}
