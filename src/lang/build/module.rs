use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::ptr::NonNull;

use super::super::ast::AST;
use super::super::parser;
use super::super::sym::{Module, Struct};
use super::super::util::{IItemPath, ItemPathBuf};
use super::super::XicCfg;
use super::{ClassBuilder, CrateBuilder};

pub struct ModuleBuilder {
    pub use_map: HashMap<String, ItemPathBuf>,
    pub class_builders: Vec<Box<ClassBuilder>>,
}

impl ModuleBuilder {
    pub fn dump(&self, cfg: &XicCfg) {
        unimplemented!()
    }
}

pub fn new_module(mod_path: ItemPathBuf, krate_builder: &mut CrateBuilder, cfg: &XicCfg) {
    let mut output_dir = cfg.out_dir.clone();
    let mut input_dir = cfg.root_dir.clone();
    let fpath = if mod_path.len() == 1 {
        // for root module, fpath is specified in cfg
        cfg.root_path.clone()
    } else {
        for (seg_id, _) in mod_path.iter().skip(1).take(mod_path.len() - 2) {
            output_dir.push(seg_id);
            input_dir.push(seg_id);
        }
        let fpath1 = input_dir.join(format!("{}.xi", mod_path.get_self().unwrap().0));
        let mut fpath2 = input_dir.join(mod_path.get_self().unwrap().0);
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
    let mut module_builder = Box::new(ModuleBuilder {
        use_map: HashMap::new(),
        class_builders: Vec::new(),
    });

    // Parse source file
    let ast = parser::parse(&fpath).unwrap();

    if cfg.verbose >= 2 {
        // save ast to .json file
        let mut f = fs::File::create(output_dir.join(format!("{}.ast.json", this_mod.self_name())))
            .unwrap();
        write!(f, "{}", ast).unwrap();
    }

    let (mods, classes) = if let AST::File(mods, classes) = *ast {
        (mods, classes)
    } else {
        unreachable!()
    };

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

        new_module(sub_mod_path, krate_builder, cfg);
    }

    // generate all classes
    for class in classes.into_iter() {
        match class.as_ref() {
            AST::Struct(ty) => {
                if this_mod.sub_mods.contains(&ty.name) {
                    panic!(
                        "Ambiguous name {} in module {}. Both a sub-module and a class",
                        ty.name,
                        this_mod.fullname()
                    );
                }

                this_mod.classes.insert(
                    ty.name.to_owned(),
                    Box::new(Struct {
                        name: ty.name.to_owned(),
                        fields: HashMap::new(),
                        methods: HashMap::new(),
                        parent: NonNull::new(this_mod.as_ref() as *const Module as *mut Module)
                            .unwrap(),
                        idx: 0,
                        attrib: ty.flags,
                    }),
                );
            }
            _ => unreachable!(),
        };
        module_builder
            .class_builders
            .push(Box::new(ClassBuilder { ast: class }));
    }

    krate_builder
        .krate
        .mod_tbl
        .insert(this_mod.fullname().to_owned(), this_mod);
    krate_builder.modules.push(module_builder);
}
