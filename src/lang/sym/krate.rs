use super::Module;

use std::collections::HashMap;

pub struct Crate {
    pub crate_name: String,

    /// key: mod_fullname
    pub mod_tbl: HashMap<String, Box<Module>>,
}
