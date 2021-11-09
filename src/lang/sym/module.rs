use std::collections::{HashMap, HashSet};
use std::fmt;

use super::super::util::{IItemPath, ItemPathBuf};
use super::Struct;

/// There is no need for module to store parent module.
/// We can use module path to determine the parent module path.
pub struct Module {
    pub mod_path: ItemPathBuf,
    pub sub_mods: HashSet<String>,
    /// key: class_name
    pub structs: HashMap<String, Box<Struct>>,
}

impl Module {
    pub fn self_name(&self) -> &str {
        self.mod_path.get_self().unwrap().0
    }

    pub fn fullname(&self) -> &str {
        self.mod_path.as_str()
    }

    pub fn is_root(&self) -> bool {
        self.mod_path.len() == 1
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fullname())
    }
}
