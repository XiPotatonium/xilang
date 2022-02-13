use std::collections::HashMap;
use std::fmt;

use super::super::ast::ClassFlags;
use super::super::util::{IItemPath, ItemPathBuf};

use super::{Field, Method};

pub struct Struct {
    pub path: ItemPathBuf,

    /// key: field_name
    pub fields: HashMap<String, Box<Field>>,
    /// key: method_name, overload is not supported
    pub methods: HashMap<String, Box<Method>>,

    pub flags: ClassFlags,
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl Struct {
    pub fn name(&self) -> &str {
        self.path.get_self().unwrap()
    }
}
