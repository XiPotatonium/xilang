use xir::attrib::{MethodAttrib, MethodAttribFlag, MethodImplAttrib, PInvokeAttrib, ParamAttrib};
use xir::file::IrFile;
use xir::sig;

use super::super::exec::internal_calls::InternalCallWrapper;
use super::super::util::ptr::NonNull;
use super::{
    builtin_ty_str_desc, param_sig_str_desc, BuiltinType, ILModule, Module, Type, REF_SIZE,
};

pub struct Param {
    pub name: usize,
    pub attrib: ParamAttrib,
    pub ty: BuiltinType,
    pub offset: usize,
}

impl Param {
    pub fn new(name: usize, attrib: ParamAttrib) -> Param {
        Param {
            name,
            attrib,
            ty: BuiltinType::Unk,
            offset: 0,
        }
    }
}

pub struct MethodDesc {
    /// module where method is declared
    pub ctx: NonNull<Module>,
    /// index in the ctx.ir_file.method_tbl
    pub index: usize,

    pub parent: *const Type,
    // slot in parent.vtbl
    pub slot: usize,

    pub name: usize,

    pub attrib: MethodAttrib,
    pub impl_attrib: MethodImplAttrib,

    pub ps: Vec<Param>,
    /// size of self ref is included
    pub ps_size: usize,
    pub ret: Param,

    pub method_impl: MethodImpl,
}

impl MethodDesc {
    pub fn is_static(&self) -> bool {
        self.attrib.is(MethodAttribFlag::Static)
    }

    // FIX: instance or not?
    pub fn str_desc_with_fullname(&self, str_pool: &Vec<String>) -> String {
        let mut sig = unsafe { self.ctx.as_ref().expect_il().fullname(str_pool).to_owned() };
        if let Some(ty) = unsafe { self.parent.as_ref() } {
            sig.push_str("/");
            sig.push_str(&str_pool[ty.name]);
        }
        sig.push_str("::");
        sig.push_str(&str_pool[self.name]);
        sig.push('(');

        for p in self.ps.iter() {
            sig.push_str(&builtin_ty_str_desc(&p.ty, str_pool));
        }

        sig.push(')');

        sig
    }
}

// FIX: instance or not?
pub fn method_str_desc(str_pool: &Vec<String>, name: usize, ps: &Vec<BuiltinType>) -> String {
    let mut sig = format!("{}(", str_pool[name]);

    for p in ps.iter() {
        sig.push_str(&builtin_ty_str_desc(p, str_pool));
    }

    sig.push(')');

    sig
}

// FIX: instance or not?
pub fn method_str_desc_from_ir(ctx: &IrFile, name: u32, ps: &Vec<sig::ParamType>) -> String {
    let mut sig = format!("{}(", ctx.get_str(name));

    for p in ps.iter() {
        sig.push_str(&param_sig_str_desc(p, ctx));
    }

    sig.push(')');

    sig
}

pub enum MethodImpl {
    IL(MethodILImpl),
    Native(MethodNativeImpl),
    Runtime(MethodRuntimeImpl),
}

impl MethodImpl {
    pub fn expect_il(&self) -> &MethodILImpl {
        match self {
            MethodImpl::IL(method_impl) => method_impl,
            MethodImpl::Native(_) => panic!(),
            MethodImpl::Runtime(_) => panic!(),
        }
    }
}

pub struct MethodILImpl {
    /// MethodDef.body
    pub index: usize,
    pub locals: Vec<Local>,
    pub locals_size: usize,
}

pub struct MethodNativeImpl {
    // index of modref (dll)
    pub scope: usize,
    pub name: usize,
    pub flag: PInvokeAttrib,
}

pub struct MethodRuntimeImpl {
    pub func: InternalCallWrapper,
}

pub struct Local {
    pub ty: BuiltinType,
    pub offset: usize,
}
