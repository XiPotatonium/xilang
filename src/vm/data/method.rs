use xir::attrib::{MethodAttrib, MethodAttribFlag, MethodImplAttrib, PInvokeAttrib, ParamAttrib};
use xir::file::IrFile;
use xir::sig;

use super::super::exec::internal_calls::InternalCallWrapper;
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
    pub ctx: *const Module,

    pub parent: *const Type,
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
    pub fn init_ps_ty(&mut self, ps_ty: &Vec<sig::ParamType>, ctx: &ILModule) {
        assert_eq!(self.ps.len(), ps_ty.len());
        let mut offset = if self.is_static() { 0 } else { REF_SIZE };
        for (p, p_ty) in self.ps.iter_mut().zip(ps_ty.iter()) {
            p.ty = BuiltinType::from_param(p_ty, ctx);
            p.offset = offset;
            // no alignment
            offset += p.ty.byte_size();
        }
        self.ps_size = offset;
    }

    pub fn is_static(&self) -> bool {
        self.attrib.is(MethodAttribFlag::Static)
    }

    // FIX: instance or not?
    pub fn str_desc_with_fullname(&self, str_pool: &Vec<String>) -> String {
        let mut sig = unsafe {
            self.ctx
                .as_ref()
                .unwrap()
                .expect_il()
                .fullname(str_pool)
                .to_owned()
        };
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
    pub offset: usize,
    pub locals: Vec<Local>,
    pub locals_size: usize,
    pub insts: Vec<u8>,
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

impl MethodILImpl {
    pub fn new(insts: Vec<u8>) -> MethodILImpl {
        MethodILImpl {
            offset: 0,
            locals: vec![],
            locals_size: 0,
            insts,
        }
    }

    pub fn init_locals(&mut self, locals: &Vec<sig::InnerLocalVarType>, ctx: &ILModule) {
        assert!(self.locals.is_empty());
        let mut offset: usize = 0;
        // no alignment
        for local_ty in locals.iter() {
            let local_ty = BuiltinType::from_local(local_ty, ctx);
            let local_size = local_ty.byte_size();
            self.locals.push(Local {
                ty: local_ty,
                offset,
            });
            offset += local_size;
        }
        self.locals_size = offset;
    }
}
