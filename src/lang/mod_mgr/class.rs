use std::collections::HashMap;

use super::Field;
use super::Method;

pub struct Class {
    pub name: String,

    /// index in module file
    pub idx: u32,

    pub non_static_fields: Vec<String>,
    pub fields: HashMap<String, Box<Field>>,
    /// overload is not allowed
    pub methods: HashMap<String, Box<Method>>,
}

impl Class {
    pub fn new(name: String, idx: u32) -> Class {
        Class {
            name,
            idx,
            non_static_fields: Vec::new(),
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }
}
