mod class;
mod field;
mod method;
mod module;
mod var;

pub use self::class::Class;
pub use self::field::Field;
pub use self::method::Method;
pub use self::module::Module;
pub use self::var::{Arg, Locals, Var};

use super::super::XicCfg;
use crate::ir::path::ModPath;

use std::fs;
use std::rc::{Rc, Weak};
use std::{collections::HashMap, path::PathBuf};

pub struct Crate {
    pub root: Rc<Module>,
    pub name: String,

    pub mod_tbl: HashMap<String, Weak<Module>>,
    // TODO Dependencies
}

impl Crate {
    pub fn new(cfg: &XicCfg) -> Crate {
        let mut mod_path: ModPath = ModPath::new();
        mod_path.push(&cfg.crate_name);

        // TODO external module paths
        println!("External module paths: {}", cfg.ext_paths.join(";"));

        // prepare output dir
        if cfg.out_dir.exists() {
            if !cfg.out_dir.is_dir() {
                panic!(
                    "{} already exists but it is not a directory",
                    cfg.out_dir.display()
                );
            }
        } else {
            fs::create_dir_all(&cfg.out_dir).unwrap();
        }

        let mut mod_tbl = HashMap::new();
        let root = Module::from_xi(
            mod_path,
            &PathBuf::from(""),
            &cfg.root_path,
            false,
            &mut mod_tbl,
            cfg,
        );
        mod_tbl.insert(cfg.crate_name.to_owned(), Rc::downgrade(&root));

        Crate {
            name: cfg.crate_name.to_owned(),
            root,
            mod_tbl,
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

    pub fn dump(&self, cfg: &XicCfg) {
        self.root.dump(&cfg.out_dir);
    }
}