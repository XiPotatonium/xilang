use std::collections::HashMap;
use std::fmt;

use xir::attrib::{FieldAttribFlag, MethodAttribFlag, TypeAttrib};

use super::{Field, Method, ModRef};

pub struct Class {
    pub parent: *const ModRef,

    pub name: String,

    /// key: field_name
    pub fields: HashMap<String, Box<Field>>,
    /// Overload is currently not supported
    ///
    /// key: method_name
    pub methods: HashMap<String, Vec<Box<Method>>>,

    pub attrib: TypeAttrib,

    pub extends: Option<*const Class>,

    /// index into typedef tbl
    pub idx: u32,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}::{}",
            unsafe { self.parent.as_ref().unwrap().fullname() },
            self.name
        )
    }
}

impl Class {
    pub fn query_method(&self, name: &str) -> Vec<&Method> {
        // TODO: check access flag
        let mut ret = Vec::new();
        let mut c = self;
        let mut is_self = true;
        loop {
            if let Some(ms) = c.methods.get(name) {
                for m in ms.iter() {
                    if (is_self || !m.attrib.is(MethodAttribFlag::Static))
                        && !m.attrib.is(MethodAttribFlag::Priv)
                    {
                        // static method cannot be accessed from derived class
                        // priv cannot be accessed from derived class
                        ret.push(m.as_ref());
                    }
                }
                break;
            }
            if let Some(base) = c.extends {
                is_self = false;
                unsafe {
                    c = base.as_ref().unwrap();
                }
            } else {
                break;
            }
        }
        ret
    }

    pub fn query_field(&self, name: &str) -> Option<&Field> {
        // TODO: check access flag
        let mut c = self;
        let mut is_self = true;
        loop {
            if let Some(f) = c.fields.get(name) {
                if is_self || !f.attrib.is(FieldAttribFlag::Static) {
                    // static field cannot be accessed from derived class
                    return Some(f.as_ref());
                }
            }
            if let Some(base) = c.extends {
                is_self = false;
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
