use super::super::native::VMDll;
use super::{VMField, VMMethod, VMType};

pub enum VMMemberRef {
    Field(*const VMField),
    Method(*const VMMethod),
}

impl VMMemberRef {
    pub fn expect_field(&self) -> *const VMField {
        if let Self::Field(f) = self {
            *f
        } else {
            panic!();
        }
    }

    pub fn expect_method(&self) -> *const VMMethod {
        if let Self::Method(m) = self {
            *m
        } else {
            panic!();
        }
    }
}

pub enum VMModule {
    IL(VMILModule),
    Native(VMDll),
}

impl VMModule {
    pub fn expect_il(&self) -> &VMILModule {
        match self {
            VMModule::IL(module) => module,
            VMModule::Native(_) => panic!(),
        }
    }

    pub fn expect_il_mut(&mut self) -> &mut VMILModule {
        match self {
            VMModule::IL(module) => module,
            VMModule::Native(_) => panic!(),
        }
    }

    pub fn expect_dll(&self) -> &VMDll {
        match self {
            VMModule::IL(_) => panic!(),
            VMModule::Native(dll) => dll,
        }
    }
}

pub struct VMILModule {
    pub modref: Vec<*const VMModule>,

    /// name -> class idx
    pub classes: Vec<Box<VMType>>,
    pub classref: Vec<*const VMType>,

    pub methods: Vec<Box<VMMethod>>,
    pub fields: Vec<Box<VMField>>,
    pub memberref: Vec<VMMemberRef>,
}
