use xir::attrib::{MethodAttrib, MethodAttribFlag, MethodImplAttrib, PInvokeAttrib, ParamAttrib};
use xir::blob::EleType;

use super::{BuiltinType, ILModule, Module, Type, REF_SIZE};

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

pub struct Method {
    /// module where method is declared
    pub ctx: *const Module,

    pub parent: *const Type,

    pub name: usize,

    pub attrib: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    pub ps: Vec<Param>,
    /// size of self ref is included
    pub ps_size: usize,
    pub ret: Param,

    pub method_impl: MethodImpl,
}

impl Method {
    pub fn init_ps_ty(&mut self, ps_ty: &Vec<EleType>, ctx: &ILModule) {
        assert_eq!(self.ps.len(), ps_ty.len());
        let mut offset = if self.is_static() { 0 } else { REF_SIZE };
        for (p, p_ty) in self.ps.iter_mut().zip(ps_ty.iter()) {
            p.ty = BuiltinType::from_ir_ele_ty(p_ty, ctx);
            p.offset = offset;
            // no alignment
            offset += p.ty.byte_size();
        }
        self.ps_size = offset;
    }

    pub fn is_static(&self) -> bool {
        self.attrib.is(MethodAttribFlag::Static)
    }
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

    pub fn init_locals(&mut self, locals: &Vec<EleType>, ctx: &ILModule) {
        assert!(self.locals.is_empty());
        let mut offset: usize = 0;
        // no alignment
        for local_ty in locals.iter() {
            let local_ty = BuiltinType::from_ir_ele_ty(local_ty, ctx);
            let local_size = local_ty.byte_size();
            self.locals.push(Local {
                ty: local_ty,
                offset,
            });
            offset += local_size;
        }
        self.locals_size = offset;
    }

    pub fn alloc_locals(&self) -> Vec<u8> {
        vec![0; self.locals_size]
    }
}
