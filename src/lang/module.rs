use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

use super::ast::ast::AST;
use super::class::Class;
use super::module_mgr::ModuleMgr;
use super::parser::peg_parser;

pub struct Module {
    name: String,
    sub_modules: HashMap<String, Rc<Module>>,
    classes: HashMap<String, Rc<Class>>,
    from_dir: bool,
}

lazy_static! {
    // Same as identifier
    static ref NAME_RULE : regex::Regex = regex::Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
}

// TODO: use regex to check module name
fn check_module_name_validity(name: &str) -> bool {
    NAME_RULE.is_match(name)
}

fn parse(
    module_path: &Vec<String>,
    paths: &Vec<PathBuf>,
    class_tbl: &mut HashMap<String, Weak<Class>>,
    show_ast: bool,
) -> HashMap<String, Rc<Class>> {
    let mut ret: HashMap<String, Rc<Class>> = HashMap::new();
    for path in paths.iter() {
        let ast = peg_parser::parse(path).unwrap();

        if show_ast {
            // save ast to .json file
            let mut f = PathBuf::from(path);
            f.set_extension("ast");
            let mut f = fs::File::create(f).unwrap();
            write!(f, "{}", ast);
        }

        if let AST::File(classes) = *ast {
            for class in classes {
                let class = Rc::new(Class::new(module_path, class));
                class_tbl.insert(class.descriptor.clone(), Rc::downgrade(&class));
                ret.insert(class.path.last().unwrap().clone(), class);
            }
        }
    }
    ret
}

impl Module {
    /// Create a module from files
    fn new(
        module_path: Vec<String>,
        paths: &Vec<PathBuf>,
        class_tbl: &mut HashMap<String, Weak<Class>>,
        save_json: bool,
    ) -> Rc<Module> {
        let classes = parse(&module_path, paths, class_tbl, save_json);
        Rc::new(Module {
            sub_modules: HashMap::new(),
            name: String::from(module_path.last().unwrap()),
            classes: classes,
            from_dir: false,
        })
    }

    /// Create a module from directory
    pub fn new_dir(
        module_path: Vec<String>,
        dir: &Path,
        class_tbl: &mut HashMap<String, Weak<Class>>,
        show_ast: bool,
    ) -> Option<Rc<Module>> {
        let mut files: Vec<PathBuf> = Vec::new();
        let mut leaf_sub_modules: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let mut ret = Module {
            sub_modules: HashMap::new(),
            name: String::from(module_path.last().unwrap()),
            classes: HashMap::new(),
            from_dir: true,
        };

        // This is a non-leaf (directory) module. Find all Mod\.(.*\.)?xi
        for entry in dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let file_name = entry.file_name().into_string().unwrap();
            let file_ty = entry.file_type().unwrap();

            if file_ty.is_dir() && check_module_name_validity(&file_name) {
                // might be a non-leaf sub-module
                let mut sub_module_path = module_path.to_vec();
                sub_module_path.push(file_name.clone());
                let sub = Module::new_dir(sub_module_path, &entry_path, class_tbl, show_ast);
                if let Some(sub) = sub {
                    // TODO: use expect_none once it is stable
                    if let Some(_) = ret.sub_modules.insert(file_name.clone(), sub) {
                        panic!(
                            "Duplicate module {} in {}",
                            file_name,
                            dir.to_str().unwrap()
                        );
                    }
                }
            } else if file_ty.is_file() && file_name.ends_with(".xi") {
                // find a .xi file
                if file_name.starts_with("Mod.") {
                    // current module file detected
                    files.push(entry_path);
                } else {
                    // leaf sub-module find
                    // TODO: use split_once() if it becomes stable
                    let module_name = entry_path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .split('.')
                        .next()
                        .unwrap();
                    if check_module_name_validity(module_name) {
                        if leaf_sub_modules.contains_key(module_name) {
                            leaf_sub_modules
                                .get_mut(module_name)
                                .unwrap()
                                .push(entry_path);
                        } else {
                            leaf_sub_modules.insert(String::from(module_name), vec![entry_path]);
                        }
                    }
                }
            }
        }

        for (leaf_sub_module_name, file_paths) in leaf_sub_modules.iter() {
            if ret.sub_modules.contains_key(leaf_sub_module_name) {
                panic!(
                    "Duplicate module {} in {}",
                    leaf_sub_module_name,
                    dir.to_str().unwrap()
                );
            } else {
                let mut sub_module_path = module_path.to_vec();
                sub_module_path.push(String::from(leaf_sub_module_name));
                let sub = Module::new(sub_module_path, file_paths, class_tbl, show_ast);
                ret.sub_modules
                    .insert(String::from(leaf_sub_module_name), sub);
            }
        }

        ret.classes = parse(&module_path, &files, class_tbl, show_ast);

        if ret.classes.len() == 0 && ret.sub_modules.len() == 0 {
            return None;
        }
        Some(Rc::new(ret))
    }

    /// Display module and its sub-modules
    pub fn tree(&self, depth: usize) {
        if depth > 0 {
            print!("{}+---", "|   ".repeat(depth - 1));
        }
        println!("{}", self.name);
        for (_, sub) in self.sub_modules.iter() {
            sub.tree(depth + 1);
        }
    }

    pub fn member_pass(&self, mgr: &ModuleMgr) {
        for class in self.classes.values() {
            class.member_pass(mgr);
        }
        for sub in self.sub_modules.values() {
            sub.member_pass(mgr);
        }
    }

    pub fn code_gen(&self, mgr: &ModuleMgr) {
        for class in self.classes.values() {
            class.code_gen(mgr);
        }
        for sub in self.sub_modules.values() {
            sub.code_gen(mgr);
        }
    }

    pub fn dump(&self, dir: PathBuf) {
        if self.from_dir {
            // create a new dir
            let mut dir = dir;
            dir.push(&self.name);
            if !dir.exists() {
                fs::create_dir(&dir).unwrap();
            } else if !dir.is_dir() {
                panic!(
                    "{} already exists but it is not a directory",
                    dir.to_str().unwrap()
                );
            }

            for class in self.classes.values() {
                class.dump(&dir);
            }
        } else {
            // dump in this dir
            for class in self.classes.values() {
                class.dump(&dir);
            }
        }
    }
}
