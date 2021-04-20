use super::super::native::VMDll;
use super::{Field, Method, Type};

pub enum MemberRef {
    Field(*const Field),
    Method(*const Method),
}

impl MemberRef {
    pub fn expect_field(&self) -> *const Field {
        if let Self::Field(f) = self {
            *f
        } else {
            panic!();
        }
    }

    pub fn expect_method(&self) -> *const Method {
        if let Self::Method(m) = self {
            *m
        } else {
            panic!();
        }
    }
}

pub enum Module {
    IL(ILModule),
    Native(VMDll),
}

impl Module {
    pub fn expect_il(&self) -> &ILModule {
        match self {
            Module::IL(module) => module,
            Module::Native(_) => panic!(),
        }
    }

    pub fn expect_il_mut(&mut self) -> &mut ILModule {
        match self {
            Module::IL(module) => module,
            Module::Native(_) => panic!(),
        }
    }

    pub fn expect_dll(&self) -> &VMDll {
        match self {
            Module::IL(_) => panic!(),
            Module::Native(dll) => dll,
        }
    }
}

pub struct ILModule {
    pub modref: Vec<*const Module>,

    /// name -> class idx
    pub classes: Vec<Box<Type>>,
    pub classref: Vec<*const Type>,

    pub methods: Vec<Box<Method>>,
    pub fields: Vec<Box<Field>>,
    pub memberref: Vec<MemberRef>,
}
