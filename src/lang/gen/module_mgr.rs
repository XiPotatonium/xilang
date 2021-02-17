use super::class::Class;
use super::module::Module;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

pub struct ModuleMgr {
    root: Rc<Module>,

    pub class_table: HashMap<String, Weak<Class>>,
    // TODO Dependencies
}

impl ModuleMgr {
    pub fn new(root_path: &PathBuf, libs: &Vec<String>, show_ast: bool) -> ModuleMgr {
        let root_path =
            fs::canonicalize(root_path).expect(&format!("Fail to canonicalize {:?}", root_path));
        let crate_name = root_path.file_name().unwrap().to_str().unwrap().to_owned();

        // TODO additional class path
        println!("Additional class path: {}", libs.join(";"));
        let mut class_tbl: HashMap<String, Weak<Class>> = HashMap::new();

        ModuleMgr {
            root: Module::new_dir(vec![crate_name], &root_path, &mut class_tbl, show_ast).unwrap(),
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

    pub fn dump(&self, out_dir: &PathBuf) {
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

        self.root.dump(out_dir.clone());
    }
}
