mod class;
mod field;
mod func;
mod module;

use self::class::ClassBuilder;
use self::field::FieldBuilder;
use self::func::FuncBuilder;
use self::module::ModuleBuilder;

use super::sym::{Module, TypeLinkContext};
use super::{XiCfg, SYS_NAME, SYS_PATH};
use core::util::ItemPathBuf;
use std::collections::HashMap;
use std::ptr::{null_mut, NonNull};

pub struct FileLoader {
    /// crates[0] is entry
    pub crates: Vec<Box<Module>>,
    /// fullname -> builder
    pub module_map: HashMap<String, NonNull<Module>>,
    pub builders: Vec<Box<ModuleBuilder>>,
}

impl FileLoader {
    pub fn load(cfg: &XiCfg) -> Box<FileLoader> {
        let mut loader = Box::new(FileLoader {
            crates: Vec::new(),
            module_map: HashMap::new(),
            builders: Vec::new(),
        });

        let entry = ModuleBuilder::load(
            ItemPathBuf::from_ir_path(cfg.entry_path.file_stem().unwrap().to_str().unwrap()),
            cfg.entry_path.clone(),
            loader.as_mut(),
            cfg,
        );
        loader.crates.push(entry);
        let loader_nonnull = NonNull::new(loader.as_mut() as *mut FileLoader).unwrap();

        // load sys
        let sys_path = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(SYS_PATH);
        println!("{}", sys_path.display());
        let syslib = ModuleBuilder::load(
            ItemPathBuf::from_ir_path(SYS_NAME),
            sys_path,
            loader.as_mut(),
            cfg,
        );
        loader.crates.push(syslib);

        // link type
        for module in loader.builders.iter_mut() {
            module.link_type(TypeLinkContext {
                loader: loader_nonnull,
                module: module.sym,
                class: null_mut(),
            });
        }

        loader
    }

    pub fn build(&mut self, cfg: &XiCfg) {
        for module in self.builders.iter_mut() {
            module.code_gen(cfg);
        }
    }

    pub fn dump(&self, cfg: &XiCfg) {
        for module in self.builders.iter() {
            module.dump(cfg);
        }
    }
}
