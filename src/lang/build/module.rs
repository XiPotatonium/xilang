use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::ptr::NonNull;

use super::super::ast::AST;
use super::super::parser;
use super::super::sym::{Module, Struct};
use super::super::util::{IItemPath, ItemPathBuf};
use super::super::XiCfg;
use super::{CrateBuilder, StructBuilder};

pub struct ModuleBuilder {
    pub parent: NonNull<CrateBuilder>,
    pub module: NonNull<Module>,
    pub use_map: HashMap<String, ItemPathBuf>,
    strukt_builders: Vec<Box<StructBuilder>>,
}

pub fn new_module(mod_path: ItemPathBuf, krate_builder: &mut CrateBuilder, cfg: &XiCfg) {
    let mut output_dir = cfg.out_dir.clone();
    let mut input_dir = cfg.root_dir.clone();
    let fpath = if mod_path.len() == 1 {
        // for root module, fpath is specified in cfg
        cfg.root_path.clone()
    } else {
        for seg_id in mod_path.iter().skip(1).take(mod_path.len() - 2) {
            output_dir.push(seg_id);
            input_dir.push(seg_id);
        }
        let fpath1 = input_dir.join(format!("{}.xi", mod_path.get_self().unwrap()));
        let mut fpath2 = input_dir.join(mod_path.get_self().unwrap());
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
        mod_path: mod_path.clone(),
        sub_mods: HashSet::new(),
        structs: HashMap::new(),
    });
    let mut module_builder = Box::new(ModuleBuilder {
        parent: NonNull::new(krate_builder as *mut CrateBuilder).unwrap(),
        module: NonNull::new(this_mod.as_ref() as *const Module as *mut Module).unwrap(),
        use_map: HashMap::new(),
        strukt_builders: Vec::new(),
    });

    // Parse source file
    let ast = parser::parse(&fpath).unwrap();

    if cfg.dump_ast {
        // save ast to .json file
        let mut f = fs::File::create(output_dir.join(format!("{}.ast.json", this_mod.self_name())))
            .unwrap();
        write!(f, "{}", ast).unwrap();
    }

    let (mods, strukts) = if let AST::File(mods, classes) = *ast {
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

        let mut sub_mod_path = mod_path.clone();
        sub_mod_path.push(&sub_mod_name);

        new_module(sub_mod_path, krate_builder, cfg);
    }

    // generate all classes
    for strukt in strukts.into_iter() {
        let strukt_sym = match strukt.as_ref() {
            AST::Struct(strukt_ast) => {
                if this_mod.sub_mods.contains(&strukt_ast.name) {
                    panic!(
                        "Ambiguous name {} in module {}. Both a sub-module and a class",
                        strukt_ast.name,
                        this_mod.fullname()
                    );
                }

                let mut struct_path = mod_path.clone();
                struct_path.push(&strukt_ast.name);
                Box::new(Struct {
                    path: struct_path,
                    fields: HashMap::new(),
                    methods: HashMap::new(),
                    flags: strukt_ast.flags,
                })
            }
            _ => unreachable!(),
        };
        module_builder
            .strukt_builders
            .push(Box::new(StructBuilder::new(
                NonNull::new(module_builder.as_ref() as *const ModuleBuilder as *mut ModuleBuilder)
                    .unwrap(),
                NonNull::new(strukt_sym.as_ref() as *const Struct as *mut Struct).unwrap(),
                strukt,
            )));
        this_mod
            .structs
            .insert(strukt_sym.name().to_owned(), strukt_sym);
    }

    krate_builder
        .krate
        .mod_tbl
        .insert(this_mod.fullname().to_owned(), this_mod);
    krate_builder.modules.push(module_builder);
}

impl ModuleBuilder {
    pub fn member_pass(&mut self) {
        for strukt_builder in self.strukt_builders.iter_mut() {
            strukt_builder.member_pass();
        }
    }

    pub fn code_gen(&mut self, cfg: &XiCfg) {
        for strukt_builder in self.strukt_builders.iter_mut() {
            strukt_builder.code_gen(cfg);
        }
    }

    pub fn dump(&self, cfg: &XiCfg) {
        for strukt_builder in self.strukt_builders.iter() {
            strukt_builder.dump(cfg);
        }
    }
}
