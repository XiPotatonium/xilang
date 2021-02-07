use super::ast::ast::AST;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::class::Class;
use super::parser::peg_parser;

pub struct Module {
    pub sub_modules: HashMap<String, Box<Module>>,
    // relative path to root module
    pub path: Vec<String>,
    pub ast: Vec<Box<AST>>,
    pub classes: HashMap<String, Box<Class>>,
}

lazy_static! {
    // Same as identifier
    static ref NAME_RULE : regex::Regex = regex::Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
}

// TODO: use regex to check module name
fn check_module_name_validity(name: &str) -> bool {
    NAME_RULE.is_match(name)
}

fn parse(paths: &Vec<PathBuf>, show_ast: bool) -> Vec<Box<AST>> {
    paths
        .iter()
        .map(|path| {
            let ast = peg_parser::parse(path).unwrap();

            if show_ast {
                // save ast to .json file
                let mut f = PathBuf::from(path);
                f.set_extension("ast");
                let mut f = fs::File::create(f).unwrap();
                write!(f, "{}", ast);
            }

            ast
        })
        .collect()
}

impl Module {
    /// Create a module from files
    fn new(paths: &Vec<PathBuf>, path: Vec<String>, save_json: bool) -> Box<Module> {
        Box::new(Module {
            sub_modules: HashMap::new(),
            path,
            ast: parse(paths, save_json),
            classes: HashMap::new(),
        })
    }

    /// Create a module from directory
    pub fn new_dir(dir: &Path, path: Vec<String>, show_ast: bool) -> Option<Box<Module>> {
        let mut files: Vec<PathBuf> = Vec::new();
        let mut leaf_sub_modules: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let mut ret: Box<Module> = Box::new(Module {
            sub_modules: HashMap::new(),
            path: path.to_vec(),
            ast: vec![],
            classes: HashMap::new(),
        });

        // This is a non-leaf (directory) module. Find all Mod\.(.*\.)?xi
        for entry in dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let file_name = entry.file_name().into_string().unwrap();
            let file_ty = entry.file_type().unwrap();

            if file_ty.is_dir() && check_module_name_validity(&file_name) {
                // might be a non-leaf sub-module
                let mut sub_path = path.to_vec();
                sub_path.push(file_name.clone());
                let sub = Module::new_dir(&entry_path, sub_path, show_ast);
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

        for (leaf_sub_module_name, leaf_sub_module_paths) in leaf_sub_modules.iter() {
            if ret.sub_modules.contains_key(leaf_sub_module_name) {
                panic!(
                    "Duplicate module {} in {}",
                    leaf_sub_module_name,
                    dir.to_str().unwrap()
                );
            } else {
                let mut sub_path = path.to_vec();
                sub_path.push(String::from(leaf_sub_module_name));
                let sub = Module::new(leaf_sub_module_paths, sub_path, show_ast);
                ret.sub_modules
                    .insert(String::from(leaf_sub_module_name), sub);
            }
        }

        ret.ast = parse(&files, show_ast);

        if ret.ast.len() == 0 && ret.sub_modules.len() == 0 {
            return None;
        }
        Some(ret)
    }

    /// Display module and its sub-modules
    pub fn tree(&self, depth: usize) {
        if depth > 0 {
            print!("{}+---", "|   ".repeat(depth - 1));
        }
        println!("{}", self.path.last().unwrap());
        for (_, sub) in self.sub_modules.iter() {
            sub.tree(depth + 1);
        }
    }

    /// Class pass
    ///
    /// Generate all classes in this module
    pub fn class_pass(&mut self) {
        for f in self.ast.iter() {
            if let AST::File(classes) = f.as_ref() {
                for c in classes.iter() {
                    if let AST::Class(id, _, _) = c.as_ref() {
                        self.classes
                            .insert(id.clone(), Box::new(Class::new(id.clone())));
                    }
                }
            } else {
                panic!("Parser error");
            }
        }
    }

    pub fn member_pass(&mut self) {}

    pub fn code_gen(&mut self) {}
}
