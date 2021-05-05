use super::super::native::VMDll;
use super::{Field, MethodDesc, Type};

pub enum MemberRef {
    Field(*const Field),
    Method(*const MethodDesc),
}

impl MemberRef {
    pub fn expect_field(&self) -> *const Field {
        if let Self::Field(f) = self {
            *f
        } else {
            panic!();
        }
    }

    pub fn expect_method(&self) -> *const MethodDesc {
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
    pub fullname: usize,

    pub modrefs: Vec<*mut Module>,

    /// name -> Type idx
    pub types: Vec<Box<Type>>,
    pub typerefs: Vec<*mut Type>,

    pub methods: Vec<Box<MethodDesc>>,
    pub fields: Vec<Box<Field>>,
    pub memberref: Vec<MemberRef>,

    pub usr_str_heap: Vec<usize>,
}

impl ILModule {
    pub fn fullname<'h>(&self, str_pool: &'h Vec<String>) -> &'h str {
        &str_pool[self.fullname]
    }
}
