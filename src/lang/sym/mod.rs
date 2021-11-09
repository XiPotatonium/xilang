mod field;
mod krate;
mod method;
mod module;
mod strukt;
mod ty;
mod var;

use std::collections::HashMap;

pub use self::field::Field;
pub use self::krate::Crate;
pub use self::method::{method_descriptor1, method_descriptor2, Method, Param};
pub use self::module::Module;
pub use self::strukt::Struct;
pub use self::ty::{RValType, SymType, ValExpectation, ValType};
pub use self::var::{Locals, Var};

pub struct SymTable<'c> {
    krate: &'c Crate,
    class: &'c Struct,
    locals: Vec<Var>,
    local_map: HashMap<String, usize>,
}

impl<'c> SymTable<'c> {
    pub fn new(krate: &'c Crate, class: &'c Struct) -> SymTable<'c> {
        SymTable {
            krate,
            class,
            locals: Vec::new(),
            local_map: HashMap::new(),
        }
    }
}
