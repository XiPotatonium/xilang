use std::collections::HashMap;
use std::fmt;

use core::flags::ClassFlags;
use core::util::{IItemPath, ItemPathBuf};
use std::ptr::NonNull;

use super::{Field, Func, Module};

pub struct Class {
    pub parent: NonNull<Module>,
    pub path: ItemPathBuf,
    pub flags: ClassFlags,

    pub impls: Vec<NonNull<Class>>,

    /// key: field_name
    pub fields: HashMap<String, Box<Field>>,
    /// key: method_name, overload is not supported
    pub methods: HashMap<String, Box<Func>>,
}

impl Class {
    pub fn fullname(&self) -> &str {
        self.path.as_str()
    }

    pub fn name(&self) -> &str {
        self.path.get_self().unwrap()
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fullname())
    }
}
