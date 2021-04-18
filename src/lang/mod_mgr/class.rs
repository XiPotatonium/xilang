use std::collections::HashMap;

use xir::attrib::TypeAttrib;

use super::Field;
use super::Method;

pub struct Class {
    pub name: String,

    // TODO: delete instance_fields, we don't need this optimization, iterate over fields is fast enough
    /// Used in new expr
    pub instance_fields: Vec<String>,
    /// key: field_name
    pub fields: HashMap<String, Box<Field>>,
    /// Overload is currently not supported
    ///
    /// key: method_name
    pub methods: HashMap<String, Box<Method>>,

    pub flag: TypeAttrib,

    /// index into typedef tbl
    pub idx: u32,
}

impl Class {
    pub fn new(name: String, idx: u32, flag: TypeAttrib) -> Class {
        Class {
            name,
            idx,
            instance_fields: Vec::new(),
            fields: HashMap::new(),
            methods: HashMap::new(),
            flag,
        }
    }
}
