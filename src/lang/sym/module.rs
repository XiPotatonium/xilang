use std::collections::HashMap;
use std::fmt;

use super::{Class, Func, Symbol};
use core::util::{IItemPath, ItemPathBuf};

/// There is no need for module to store parent module.
/// We can use module path to determine the parent module path.
pub struct Module {
    pub path: ItemPathBuf,
    pub sub_mods: HashMap<String, Box<Module>>,
    pub use_map: HashMap<String, Symbol>,
    /// key: class_name
    pub classes: HashMap<String, Box<Class>>,
    /// key: function name, overload not allowed
    pub funcs: HashMap<String, Box<Func>>,
}

impl Module {
    pub fn name(&self) -> &str {
        self.path.get_self().unwrap()
    }

    pub fn fullname(&self) -> &str {
        self.path.as_str()
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fullname())
    }
}
