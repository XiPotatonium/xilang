use super::super::XicCfg;
use super::class::Class;
use super::module::Module;
use crate::ir::path::ModPath;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};

pub struct Crate {
    pub root: Rc<Module>,
    pub name: String,

    pub class_tbl: HashMap<String, Weak<RefCell<Class>>>,
    // TODO Dependencies
}

impl Crate {
    pub fn new(cfg: &XicCfg) -> Crate {
        let mut mod_path: ModPath = ModPath::new();
        mod_path.push(&cfg.crate_name);

        // TODO external module paths
        println!("External module paths: {}", cfg.ext_paths.join(";"));

        let mut class_tbl = HashMap::new();
        let root = Module::new(
            mod_path,
            &cfg.root_path,
            true,
            false,
            &mut class_tbl,
            cfg.verbose >= 2,
        );

        Crate {
            name: cfg.crate_name.to_owned(),
            root,
            class_tbl,
        }
    }

    // Print crate structure like a tree
    pub fn tree(&self) {
        self.root.tree(0);
    }

    pub fn build(&mut self) {
        // 1. member pass
        self.root.member_pass(self);

        // 2. code gen
        self.root.code_gen(self);
    }

    pub fn dump(&self, out_dir: &Path) {
        if out_dir.exists() {
            if !out_dir.is_dir() {
                panic!(
                    "{} already exists but it is not a directory",
                    out_dir.to_str().unwrap()
                );
            }
        } else {
            fs::create_dir_all(out_dir).unwrap();
        }

        self.root.dump(&out_dir);
    }
}
