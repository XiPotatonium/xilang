use super::ast::ast::AST;
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

use super::parser::ll_parser;

pub struct Module {
    pub parent: Option<Weak<RefCell<Module>>>,
    pub sub_modules: HashMap<String, Rc<RefCell<Module>>>,
    pub name: String,
    pub ast: Vec<AST>,
}

lazy_static! {
    // Same as identifier
    static ref NAME_RULE : regex::Regex = regex::Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
}

// TODO: use regex to check module name
fn check_module_name_validity(name: &str) -> bool {
    NAME_RULE.is_match(name)
}

fn parse(paths: &Vec<PathBuf>, save_json: bool) -> Vec<AST> {
    let mut ret: Vec<AST> = Vec::new();

    for path in paths.iter() {
        let mut parser = ll_parser::LLParser::new(fs::File::open(path).unwrap());
        let ast = parser.parse().unwrap();

        if save_json {
            // save ast to .json file
            let mut f = PathBuf::from(path);
            f.set_extension(".ast.json");
            let mut f = fs::File::create(f).unwrap();
            write!(f, "{}", ast);
        }

        ret.push(ast);
    }

    ret
}

impl Module {
    fn new_leaf(
        parent: Option<Weak<RefCell<Module>>>,
        paths: &Vec<PathBuf>,
        name: String,
        save_json: bool
    ) -> Rc<RefCell<Module>> {
        Rc::new(RefCell::new(Module {
            parent,
            sub_modules: HashMap::new(),
            name,
            ast: parse(paths, save_json),
        }))
    }

    pub fn new_non_leaf(
        parent: Option<Weak<RefCell<Module>>>,
        dir: &Path,
        name: String,
        save_json: bool
    ) -> Option<Rc<RefCell<Module>>> {
        let mut files: Vec<PathBuf> = Vec::new();
        let mut leaf_sub_modules: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let ret: Rc<RefCell<Module>> = Rc::new(RefCell::new(Module {
            parent,
            sub_modules: HashMap::new(),
            name,
            ast: vec![],
        }));

        {
            let mut ret_module_ref = ret.borrow_mut();

            // This is a non-leaf (directory) module. Find all Mod\.(.*\.)?xi
            for entry in dir.read_dir().unwrap() {
                let entry = entry.unwrap();
                let entry_path = entry.path();
                let file_name = entry.file_name().into_string().unwrap();
                let file_ty = entry.file_type().unwrap();

                if file_ty.is_dir() && check_module_name_validity(&file_name) {
                    // might be a non-leaf sub-module
                    let sub = Module::new_non_leaf(
                        Some(Rc::downgrade(&ret)),
                        &entry_path,
                        file_name.clone(),
                        save_json
                    );
                    if let Some(sub) = sub {
                        // TODO: use expect_none once it is stable
                        if let Some(_) = ret_module_ref.sub_modules.insert(file_name.clone(), sub) {
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
                                leaf_sub_modules
                                    .insert(String::from(module_name), vec![entry_path]);
                            }
                        }
                    }
                }
            }

            for (leaf_sub_module_name, leaf_sub_module_paths) in leaf_sub_modules.iter() {
                if ret_module_ref
                    .sub_modules
                    .contains_key(leaf_sub_module_name)
                {
                    panic!(
                        "Duplicate module {} in {}",
                        leaf_sub_module_name,
                        dir.to_str().unwrap()
                    );
                } else {
                    let sub = Module::new_leaf(
                        Some(Rc::downgrade(&ret)),
                        leaf_sub_module_paths,
                        String::from(leaf_sub_module_name),
                        save_json
                    );
                    ret_module_ref
                        .sub_modules
                        .insert(String::from(leaf_sub_module_name), sub);
                }
            }

            ret_module_ref.ast = parse(&files, save_json);

            if ret_module_ref.ast.len() == 0 && ret_module_ref.sub_modules.len() == 0 {
                return None;
            }
        }
        Some(ret)
    }

    pub fn tree(&self, depth: usize) {
        if depth > 0 {
            print!("{}+---", "|   ".repeat(depth - 1));
        }
        println!("{}", self.name);
        for (_, sub) in self.sub_modules.iter() {
            sub.borrow().tree(depth + 1);
        }
    }
}
