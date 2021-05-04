mod code_gen_pass;
mod member_pass;

use core::panic;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::ptr;

use xir::file::IrFile;
use xir::util::path::{IModPath, ModPath};

use super::super::ast::AST;
use super::super::gen::{Builder, RValType};
use super::super::parser;
use super::super::XicCfg;
use super::external::load_external_crate;
use super::{Class, ModMgr, ModRef};

pub struct Module {
    pub mod_path: ModPath,
    pub sub_mods: HashSet<String>,
    /// key: class_name
    pub classes: HashMap<String, RefCell<Class>>,

    /// Vec<Box<AST::Class>>
    class_asts: Vec<Box<AST>>,
    is_dir_mod: bool,
    pub use_map: HashMap<String, ModPath>,

    pub builder: RefCell<Builder>,
}

impl Module {
    /// Create a module from directory
    pub fn new(
        mod_path: ModPath,
        rel_dir: &Path,
        fpath: &Path,
        is_dir_mod: bool,
        mod_tbl: &mut HashMap<String, Box<ModRef>>,
        cfg: &XicCfg,
    ) {
        let output_dir = cfg.out_dir.join(rel_dir);
        let mod_self_name = mod_path.get_self_name().unwrap();

        let builder = Builder::new(mod_path.as_str());

        // Parse source file
        let ast = parser::peg_parse(fpath).unwrap();

        if cfg.verbose >= 2 {
            // save ast to .json file
            let mut f =
                fs::File::create(output_dir.join(format!("{}.ast.json", mod_self_name))).unwrap();
            write!(f, "{}", ast).unwrap();
        }

        if let AST::File(mods, exts, uses, classes) = *ast {
            if mod_path.len() == 1 {
                // load external modules specified in root module
                let mut exts_map: HashMap<String, Option<&PathBuf>> = HashMap::new();
                for ext in exts.iter() {
                    if let Some(_) = exts_map.insert(ext.to_owned(), None) {
                        panic!("Declaring duplicated external module {}", ext);
                    }
                }
                for ext_path in cfg.ext_paths.iter() {
                    let file = IrFile::from_binary(Box::new(fs::File::open(ext_path).unwrap()));
                    let ext_mod_name = file.mod_name();
                    // only import declared external modules
                    if let Some(imported) = exts_map.get_mut(ext_mod_name) {
                        if let Some(old_path) = imported {
                            panic!(
                                "Ambiguous external module {}: {} or {}?",
                                ext_mod_name,
                                old_path.display(),
                                ext_path.display()
                            );
                        } else {
                            *imported = Some(ext_path);
                        }
                    }
                    load_external_crate(mod_tbl, ext_path.parent().unwrap(), file);
                }
            } else if exts.len() != 0 {
                println!("Warning: {} is not root mod. External mod specified in this file won't take effect", fpath.display());
            }

            let mut use_map: HashMap<String, ModPath> = HashMap::new();
            // process sub mods
            let mut sub_mods = HashSet::new();
            let sub_mod_rel_dir = if mod_path.len() == 1 || is_dir_mod {
                // this is the root mod in this dir
                // search sub modules in this directory
                rel_dir.to_owned()
            } else {
                // this is a normal file mod
                // search sub modules in directory dir/mod_name
                rel_dir.join(mod_self_name)
            };
            // input_root/sub_mod_rel_dir
            let sub_mod_input_dir = cfg.root_dir.join(&sub_mod_rel_dir);
            // output_root/sub_mod_rel_dir
            let sub_mod_output_dir = cfg.out_dir.join(&sub_mod_rel_dir);

            for sub_mod_name in mods.into_iter() {
                if !sub_mods.insert(sub_mod_name.clone()) {
                    panic!(
                        "Sub-module {} is defined multiple times in {}",
                        sub_mod_name,
                        mod_path.as_str()
                    );
                }

                let mut sub_mod_path = mod_path.clone();
                sub_mod_path.push(&sub_mod_name);

                // sub mods will be use by default
                if use_map.contains_key(&sub_mod_name) {
                    panic!(
                        "Ambiguous id {}. Both a sub module and a using token",
                        sub_mod_name
                    );
                } else {
                    use_map.insert(sub_mod_name.clone(), sub_mod_path.clone());
                }

                // possible locations
                // * input_root/sub_mod_rel_dir/sub_mod_name.xi
                // * input_root/sub_mod_rel_dir/sub_mod_name/mod.xi
                // * input_root/sub_mod_rel_dir/sub_mod_name.xir
                // * input_root/sub_mod_rel_dir/sub_mod_name/sub_mod_name.xir

                // input_root/sub_mod_rel_dir/sub_mod_name.xi
                let sub_mod_fpath = sub_mod_input_dir.join(format!("{}.xi", sub_mod_name));
                // input_root/sub_mod_rel_dir/sub_mod_name/mod.xi
                let sub_mod_dpath = sub_mod_input_dir.join(format!("{}\\mod.xi", &sub_mod_name));

                if sub_mod_fpath.is_file() && sub_mod_dpath.is_file() {
                    panic!(
                        "Ambiguous sub-module {} in {}. {} or {}?",
                        sub_mod_name,
                        mod_path.as_str(),
                        sub_mod_dpath.display(),
                        sub_mod_fpath.display()
                    );
                } else if sub_mod_dpath.is_file() {
                    // prepare output dir for sub dir mod
                    // output_dir/sub_mod_rel_dir/sub_mod_name
                    let sub_mod_out_dir = sub_mod_output_dir.join(&sub_mod_name);
                    if !sub_mod_out_dir.exists() {
                        fs::create_dir_all(sub_mod_out_dir).unwrap();
                    } else if !sub_mod_out_dir.is_dir() {
                        panic!("Path {} is not a directory", sub_mod_out_dir.display());
                    }

                    let sub_mod_rel_dir = sub_mod_rel_dir.join(&sub_mod_name);
                    Module::new(
                        sub_mod_path,
                        &sub_mod_rel_dir,
                        &sub_mod_dpath,
                        true,
                        mod_tbl,
                        cfg,
                    );
                } else if sub_mod_fpath.is_file() {
                    Module::new(
                        sub_mod_path,
                        &sub_mod_rel_dir,
                        &sub_mod_fpath,
                        false,
                        mod_tbl,
                        cfg,
                    );
                } else {
                    panic!(
                        "Cannot find sub-module {} in {} (Consider create {} or {})",
                        sub_mod_name,
                        mod_path.as_str(),
                        sub_mod_dpath.display(),
                        sub_mod_fpath.display()
                    );
                }
            }

            // generate all classes
            let mut class_map = HashMap::new();
            for class in classes.iter() {
                if let AST::Class(class) = class.as_ref() {
                    if sub_mods.contains(&class.name) {
                        panic!(
                            "Ambiguous name {} in module {}. Both a sub-module and a class",
                            class.name, mod_self_name
                        );
                    }

                    class_map.insert(
                        class.name.to_owned(),
                        RefCell::new(Class {
                            name: class.name.to_owned(),
                            fields: HashMap::new(),
                            methods: HashMap::new(),
                            parent: ptr::null(),
                            idx: 0,
                            extends: None,
                            attrib: class.attrib.clone(),
                        }),
                    );
                } else {
                    unreachable!();
                }
            }

            // process uses
            for use_ast in uses.iter() {
                if let AST::Use(raw_path, as_id) = use_ast.as_ref() {
                    let (path_has_crate, path_super_count, can_path) = raw_path.canonicalize();

                    let use_path = if path_has_crate {
                        let mut use_path = ModPath::new();
                        use_path.push(mod_path.get_root_name().unwrap());
                        for seg in can_path.range(1, can_path.len()).iter() {
                            use_path.push(seg);
                        }
                        use_path
                    } else if path_super_count != 0 {
                        let mut root_path = mod_path.get_super();
                        for _ in (0..path_super_count).into_iter() {
                            root_path.to_super();
                        }
                        let mut use_path = root_path.to_owned();
                        for seg in can_path.range(path_super_count, can_path.len()).iter() {
                            use_path.push(seg);
                        }
                        use_path
                    } else {
                        can_path
                    };

                    let as_id = if let Some(as_id) = as_id {
                        as_id.to_owned()
                    } else {
                        use_path.get_self_name().unwrap().to_owned()
                    };

                    if use_map.contains_key(&as_id) {
                        panic!("Duplicated use as {}", as_id);
                    } else {
                        use_map.insert(as_id, use_path);
                    }

                    todo!("TODO: Validate use path");
                } else {
                    unreachable!();
                }
            }

            let this_mod = Module {
                mod_path,
                sub_mods,
                classes: class_map,
                class_asts: classes,
                is_dir_mod,
                use_map,

                builder: RefCell::new(builder),
            };
            mod_tbl.insert(
                this_mod.fullname().to_owned(),
                Box::new(ModRef::Mod(this_mod)),
            );
        } else {
            unreachable!();
        }
    }

    pub fn name(&self) -> &str {
        self.mod_path.get_self_name().unwrap()
    }

    pub fn fullname(&self) -> &str {
        self.mod_path.as_str()
    }

    pub fn is_root(&self) -> bool {
        self.mod_path.len() == 1
    }

    pub fn dump(&self, mgr: &ModMgr, out_dir: &Path) {
        let mut p = out_dir.join(format!("{}.xir", self.name()));

        // dump xir
        let mut f = fs::File::create(&p).unwrap();
        write!(f, "{}", self.builder.borrow().file).unwrap();

        p.set_extension("xibc");
        let buf = self.builder.borrow().file.to_binary();
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(&buf).unwrap();

        let out_dir = if self.is_dir_mod {
            p.set_file_name(self.name());
            if !p.exists() {
                fs::create_dir(&p).unwrap();
            } else if !p.is_dir() {
                panic!("{} already exists but it is not a directory", p.display());
            }
            &p
        } else {
            out_dir
        };

        for sub in self.sub_mods.iter() {
            let mut sub_mod_path = self.mod_path.clone();
            sub_mod_path.push(sub);
            let sub = mgr.mod_tbl.get(sub_mod_path.as_str()).unwrap().expect_mod();
            sub.dump(mgr, out_dir);
        }
    }
}

impl Module {
    /// item must exist
    pub fn resolve_path(
        &self,
        path: &ModPath,
        mod_mgr: &ModMgr,
        class: Option<&Class>,
    ) -> (String, String) {
        let (has_crate, super_cnt, path) = path.canonicalize();
        let class_id = path.get_self_name().unwrap();
        let mod_path = path.get_super();
        if mod_path.len() == 0 {
            // this mod
            // might be a class in this module
            (
                self.fullname().to_owned(),
                if class_id == "Self" {
                    if let Some(class) = class {
                        class.name.clone()
                    } else {
                        panic!("Invalid Self keyword outside a class");
                    }
                } else if self.classes.contains_key(class_id) {
                    class_id.to_owned()
                } else {
                    panic!("No class {} in mod {}", class_id, self.fullname());
                },
            )
        } else {
            let m = if has_crate {
                // crate::...
                let mut m = ModPath::new();
                m.push(&mod_mgr.cfg.crate_name);
                for seg in mod_path.iter().skip(1) {
                    m.push(seg);
                }
                m
            } else if super_cnt != 0 {
                // super::...
                let mut m = self.mod_path.as_slice();
                for _ in (0..super_cnt).into_iter() {
                    m.to_super();
                }
                let mut m = m.to_owned();
                for seg in mod_path.iter().skip(super_cnt) {
                    m.push(seg);
                }
                m
            } else {
                let mut mod_path_iter = mod_path.iter();
                let r = mod_path_iter.next().unwrap();
                if let Some(m) = self.use_map.get(r) {
                    let mut m = m.clone();
                    for seg in mod_path_iter {
                        m.push(seg);
                    }
                    m
                } else {
                    mod_path.to_owned()
                }
            };

            if let Some(m) = mod_mgr.mod_tbl.get(m.as_str()) {
                if let Some(_) = m.get_class(class_id) {
                    (m.fullname().to_owned(), class_id.to_owned())
                } else {
                    panic!("Class {} not found", class_id);
                }
            } else {
                panic!("Module {} not found", m.as_str());
            }
        }
    }

    pub fn get_ty(&self, ast: &AST, mod_mgr: &ModMgr, class: &Class) -> RValType {
        match ast {
            AST::TypeI32 => RValType::I32,
            AST::TypeF64 => RValType::F64,
            AST::TypeBool => RValType::Bool,
            AST::None => RValType::Void,
            AST::TypeTuple(_) => {
                unimplemented!();
            }
            AST::Path(class_path) => {
                let (mod_name, class_name) = self.resolve_path(class_path, mod_mgr, Some(class));
                RValType::Obj(mod_name, class_name)
            }
            AST::TypeArr(dtype, _) => RValType::Array(Box::new(self.get_ty(dtype, mod_mgr, class))),
            _ => unreachable!(),
        }
    }
}
