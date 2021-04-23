use std::collections::HashMap;
use std::fmt;

use xir::attrib::TypeAttrib;

use super::{Field, Method, ModRef};

pub struct Class {
    pub parent: *const ModRef,

    pub name: String,

    /// key: field_name
    pub fields: HashMap<String, Box<Field>>,
    /// Overload is currently not supported
    ///
    /// key: method_name
    pub methods: HashMap<String, Box<Method>>,

    pub attrib: TypeAttrib,

    pub extends: Option<*const Class>,

    /// index into typedef tbl
    pub idx: u32,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Class {
    pub fn query_method(&self, name: &str) -> Option<&Method> {
        // TODO: check access flag
        let mut c = self;
        loop {
            if let Some(m) = c.methods.get(name) {
                return Some(m.as_ref());
            }
            if let Some(base) = c.extends {
                unsafe {
                    c = base.as_ref().unwrap();
                }
            } else {
                break;
            }
        }
        None
    }

    pub fn query_field(&self, name: &str) -> Option<&Field> {
        // TODO: check access flag
        let mut c = self;
        loop {
            if let Some(f) = c.fields.get(name) {
                return Some(f.as_ref());
            }
            if let Some(base) = c.extends {
                unsafe {
                    c = base.as_ref().unwrap();
                }
            } else {
                break;
            }
        }
        None
    }
}
