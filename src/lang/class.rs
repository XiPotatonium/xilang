use std::collections::HashMap;

use crate::ir::var::VarType;

pub struct Var {
    ty: VarType,
    offset: u32,
}

pub enum Member {
    Field(Var),
    Method(VarType, Vec<Var>),
}

pub struct Class {
    name: String,
    // overload is not allowed
    members: HashMap<String, Box<Member>>,
}

impl Class {
    pub fn new(name: String) -> Class {
        Class {
            name,
            members: HashMap::new(),
        }
    }
}
