use super::{VMClass, VMField, VMMethod};

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

pub struct VMModule {
    pub modref: Vec<*const VMModule>,

    /// name -> class idx
    pub classes: Vec<Box<VMClass>>,
    pub classref: Vec<*const VMClass>,

    pub methods: Vec<Box<VMMethod>>,
    pub fields: Vec<Box<VMField>>,
    pub memberref: Vec<VMMemberRef>,
}
