use std::collections::HashMap;
use std::fmt;
use std::ptr::NonNull;

use ir::flags::ClassFlags;

use super::{Field, Method, Module};

pub struct Struct {
    pub parent: NonNull<Module>,

    pub name: String,

    /// key: field_name
    pub fields: HashMap<String, Box<Field>>,
    /// key: method_name, overload is not supported
    pub methods: HashMap<String, Box<Method>>,

    pub flags: ClassFlags,

    /// index into typedef tbl
    pub idx: u32,
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.modname(), self.name)
    }
}

impl Struct {
    pub fn modname(&self) -> &str {
        unsafe { self.parent.as_ref().fullname() }
    }
}
