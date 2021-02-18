use super::class::Class;
use super::module::Module;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::rc::{Rc, Weak};

pub struct Crate {
    pub root: Rc<Module>,

    pub class_table: HashMap<String, Weak<RefCell<Class>>>,
    // TODO Dependencies
}

impl Crate {
    pub fn new(root_path: &Path, exts: &Vec<String>, show_ast: bool) -> Crate {
        let root_name = root_path.file_stem().unwrap().to_str().unwrap().to_owned();

        // TODO external module paths
        println!("External module paths: {}", exts.join(";"));
        let mut class_tbl: HashMap<String, Weak<RefCell<Class>>> = HashMap::new();

        Crate {
            root: Module::new(vec![root_name], &root_path, false, &mut class_tbl, show_ast),
            class_table: class_tbl,
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
