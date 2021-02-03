use super::module;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub struct CrateMgr {
    target: Crate,
    // TODO Dependencies
}

impl CrateMgr {
    pub fn new(root_path: &PathBuf, save_json: bool) -> CrateMgr {
        CrateMgr {
            target: Crate::new(root_path, save_json),
        }
    }

    // Print crate structure like a tree
    pub fn tree(&self) {
        self.target.root.borrow().tree(0);
    }

    pub fn build(&self) {
        unimplemented!();
    }

    pub fn dump(&self, out_dir: &PathBuf) {
        unimplemented!();
    }
}

pub struct Crate {
    root: Rc<RefCell<module::Module>>,
}

impl Crate {
    fn new(root_path: &PathBuf, save_json: bool) -> Crate {
        let root_path =
            fs::canonicalize(root_path).expect(&format!("Fail to canonicalize {:?}", root_path));
        let crate_name = root_path.file_name().unwrap().to_str().unwrap().to_owned();
        Crate {
            root: module::Module::new_non_leaf(None, &root_path, crate_name, save_json).unwrap(),
        }
    }
}
