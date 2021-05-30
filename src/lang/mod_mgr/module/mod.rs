mod code_gen_pass;
mod member_pass;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::ptr::{self, NonNull};

use xir::file::IrFile;
use xir::util::path::{IModPath, ModPath};

use super::super::super::XicCfg;
use super::super::ast::{ASTType, AST};
use super::super::gen::{Builder, RValType};
use super::super::parser;
use super::external::load_external_crate;
use super::{Crate, Type};

pub struct Module {
    pub mod_path: ModPath,
    pub sub_mods: HashSet<String>,
    /// key: class_name
    pub classes: HashMap<String, Box<Type>>,
}

impl Module {
    pub fn self_name(&self) -> &str {
        self.mod_path.get_self_name().unwrap()
    }

    pub fn fullname(&self) -> &str {
        self.mod_path.as_str()
    }

    pub fn is_root(&self) -> bool {
        self.mod_path.len() == 1
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fullname())
    }
}

pub struct ModuleBuildCtx {
    /// Vec<Box<AST::Class|AST::Struct>>
    class_asts: Vec<Box<AST>>,
    pub use_map: HashMap<String, ModPath>,

    pub builder: RefCell<Builder>,

    // not using NonNull because we want to get mut ref from &self functin
    module: *mut Module,
}

pub fn new_module(mod_path: ModPath, mgr: &mut Crate, cfg: &XicCfg) {
    let mut output_dir = cfg.out_dir.clone();
    let mut input_dir = cfg.root_dir.clone();
    let fpath = if mod_path.len() == 1 {
        // for root module, fpath is specified in cfg
        cfg.root_path.clone()
    } else {
        for seg in mod_path.iter().skip(1).take(mod_path.len() - 2) {
            output_dir.push(seg);
            input_dir.push(seg);
        }
        let fpath1 = input_dir.join(format!("{}.xi", mod_path.get_self_name().unwrap()));
        let mut fpath2 = input_dir.join(mod_path.get_self_name().unwrap());
        fpath2.push("mod.xi");
        if fpath1.is_file() && fpath2.is_file() {
            panic!(
                "Ambiguous module {}. {} or {}?",
                mod_path,
                fpath1.display(),
                fpath2.display()
            );
        }
        if fpath1.is_file() {
            fpath1
        } else if fpath2.is_file() {
            fpath2
        } else {
            panic!(
                "Cannot find module {} (Consider create {} or {})",
                mod_path,
                fpath1.display(),
                fpath2.display()
            );
        }
    };

    fs::create_dir_all(&output_dir).unwrap();

    let mut this_mod = Box::new(Module {
        mod_path,
        sub_mods: HashSet::new(),
        classes: HashMap::new(),
    });

    // Parse source file
    let ast = parser::peg_parse(&fpath).unwrap();

    if cfg.verbose >= 2 {
        // save ast to .json file
        let mut f = fs::File::create(output_dir.join(format!("{}.ast.json", this_mod.self_name())))
            .unwrap();
        write!(f, "{}", ast).unwrap();
    }

    if let AST::File(mods, exts, uses, classes) = *ast {
        if this_mod.is_root() {
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
                load_external_crate(&mut mgr.mod_tbl, ext_path.parent().unwrap(), file);
            }
        } else if exts.len() != 0 {
            println!("Warning: {} is not root mod. External mod specified in this file won't take effect", this_mod.fullname());
        }

        for sub_mod_name in mods.into_iter() {
            if !this_mod.sub_mods.insert(sub_mod_name.clone()) {
                panic!(
                    "Sub-module {} is defined multiple times in {}",
                    sub_mod_name,
                    this_mod.fullname()
                );
            }

            let mut sub_mod_path = this_mod.mod_path.clone();
            sub_mod_path.push(&sub_mod_name);

            new_module(sub_mod_path, mgr, cfg);
        }

        // generate all classes
        for class in classes.iter() {
            match class.as_ref() {
                AST::Class(ty) | AST::Struct(ty) => {
                    if this_mod.sub_mods.contains(&ty.name) {
                        panic!(
                            "Ambiguous name {} in module {}. Both a sub-module and a class",
                            ty.name,
                            this_mod.fullname()
                        );
                    }

                    this_mod.classes.insert(
                        ty.name.to_owned(),
                        Box::new(Type {
                            name: ty.name.to_owned(),
                            fields: HashMap::new(),
                            methods: HashMap::new(),
                            parent: NonNull::new(this_mod.as_ref() as *const Module as *mut Module)
                                .unwrap(),
                            idx: 0,
                            extends: ptr::null(),
                            attrib: ty.attrib.clone(),
                        }),
                    );
                }
                _ => unreachable!(),
            }
        }

        // process uses
        let mut use_map: HashMap<String, ModPath> = HashMap::new();
        for use_ast in uses.iter() {
            if let AST::Use(raw_path, as_id) = use_ast.as_ref() {
                let (path_has_crate, path_super_count, can_path) = raw_path.canonicalize();

                let use_path = if path_has_crate {
                    let mut use_path = ModPath::new();
                    use_path.push(&mgr.crate_name);
                    for seg in can_path.range(1, can_path.len()).iter() {
                        use_path.push(seg);
                    }
                    use_path
                } else if path_super_count != 0 {
                    let mut root_path = this_mod.mod_path.get_super();
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

        let mod_build_ctx = ModuleBuildCtx {
            class_asts: classes,
            use_map,

            module: this_mod.as_mut() as *mut Module,

            builder: RefCell::new(Builder::new(this_mod.fullname())),
        };
        mgr.mod_tbl.insert(this_mod.fullname().to_owned(), this_mod);
        mgr.mod_build_ctx.push(mod_build_ctx);
    } else {
        unreachable!();
    }
}

impl ModuleBuildCtx {
    pub fn get_module(&self) -> &Module {
        unsafe { self.module.as_ref().unwrap() }
    }

    pub fn get_module_mut(&self) -> &mut Module {
        unsafe { self.module.as_mut().unwrap() }
    }

    pub fn dump(&self, cfg: &XicCfg) {
        let mut p = cfg.out_dir.clone();
        let mod_path = &self.get_module().mod_path;
        if mod_path.len() == 1 {
            // root module is output at {out_dir}/{root_name}.xibc
            p.push(mod_path.get_self_name().unwrap());
        } else {
            // other modules are output at {out_dir}/{mod_path_except_root}.xibc
            for seg in self.get_module().mod_path.iter().skip(1) {
                p.push(seg);
            }
        }
        // dump ir
        p.set_extension("xir");
        let mut f = fs::File::create(&p).unwrap();
        write!(f, "{}", self.builder.borrow().file).unwrap();

        // dump byte code
        p.set_extension("xibc");
        let buf = self.builder.borrow().file.to_binary();
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(&buf).unwrap();
    }
}

impl ModuleBuildCtx {
    /// item must exist
    pub fn resolve_user_define_type(
        &self,
        path: &ModPath,
        c: &Crate,
        class: Option<&Type>,
    ) -> NonNull<Type> {
        let (has_crate, super_cnt, canonicalized_path) = path.canonicalize();
        let class_id = canonicalized_path.get_self_name().unwrap();
        let mod_path = canonicalized_path.get_super();
        let module = self.get_module();
        if mod_path.len() == 0 {
            // this mod
            // might be a class in this module
            if class_id == "Self" {
                if let Some(class) = class {
                    NonNull::new(class as *const Type as *mut Type).unwrap()
                } else {
                    panic!("Invalid Self keyword outside a class");
                }
            } else if let Some(ty) = module.classes.get(class_id) {
                NonNull::new(ty.as_ref() as *const Type as *mut Type).unwrap()
            } else {
                panic!("No class {} in mod {}", class_id, module.fullname());
            }
        } else {
            let m = if has_crate {
                // crate::...
                let mut m = ModPath::new();
                m.push(&c.crate_name);
                for seg in mod_path.iter().skip(1) {
                    m.push(seg);
                }
                m
            } else if super_cnt != 0 {
                // super::...
                let mut m = module.mod_path.as_slice();
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
                } else if self.get_module().sub_mods.contains(r) {
                    let mut m = self.get_module().mod_path.clone();
                    m.push(r);
                    for seg in mod_path_iter {
                        m.push(seg);
                    }
                    m
                } else {
                    panic!("Cannot resolve path {}", path);
                }
            };

            if let Some(m) = c.mod_tbl.get(m.as_str()) {
                if let Some(ty) = m.classes.get(class_id) {
                    NonNull::new(ty.as_ref() as *const Type as *mut Type).unwrap()
                } else {
                    panic!("Class {} not found", class_id);
                }
            } else {
                panic!("Module {} not found", m.as_str());
            }
        }
    }

    pub fn get_rval_type(&self, ast: &ASTType, mod_mgr: &Crate, class: &Type) -> RValType {
        match ast {
            ASTType::I32 => RValType::I32,
            ASTType::F64 => RValType::F64,
            ASTType::Bool => RValType::Bool,
            ASTType::None => RValType::Void,
            ASTType::Char => RValType::Char,
            ASTType::String => RValType::String,
            ASTType::Tuple(_) => {
                unimplemented!();
            }
            ASTType::Class(class_path) => {
                RValType::Type(self.resolve_user_define_type(class_path, mod_mgr, Some(class)))
            }
            ASTType::Arr(dtype) => {
                RValType::Array(Box::new(self.get_rval_type(dtype, mod_mgr, class)))
            }
        }
    }
}
