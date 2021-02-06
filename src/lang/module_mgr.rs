use super::module;
use std::fs;
use std::path::PathBuf;

pub struct ModuleMgr {
    root: Box<module::Module>,
    // TODO Dependencies
}

impl ModuleMgr {
    pub fn new(root_path: &PathBuf, libs: &Vec<String>, save_json: bool) -> ModuleMgr {
        let root_path =
            fs::canonicalize(root_path).expect(&format!("Fail to canonicalize {:?}", root_path));
        let crate_name = root_path.file_name().unwrap().to_str().unwrap().to_owned();

        // TODO additional class path

        ModuleMgr {
            root: module::Module::new_dir(&root_path, vec![crate_name], save_json).unwrap(),
        }
    }

    // Print crate structure like a tree
    pub fn tree(&self) {
        self.root.tree(0);
    }

    pub fn build(&mut self) {
        // 1. class pass
        self.root.class_pass();

        // 2. member pass
        self.root.member_pass();

        // 3. code gen
        self.root.code_gen();
    }

    pub fn dump(&self, out_dir: &PathBuf) {
        unimplemented!();
    }
}
