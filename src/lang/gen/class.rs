use std::collections::HashMap;

use super::field::Field;
use super::method::Method;

pub struct Class {
    pub fullname: String,

    /// index in module file
    pub idx: u32,

    pub non_static_fields: Vec<String>,
    pub fields: HashMap<String, Box<Field>>,
    /// overload is not allowed
    pub methods: HashMap<String, Box<Method>>,
}

impl Class {
    pub fn new(fullname: String, idx: u32) -> Class {
        Class {
            fullname,
            idx,
            non_static_fields: Vec::new(),
            fields: HashMap::new(),
            methods: HashMap::new(),
        }
    }
}
