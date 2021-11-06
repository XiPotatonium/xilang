use std::collections::HashMap;
use std::fs;

use super::super::sym::Crate;
use super::super::util::ItemPathBuf;
use super::super::XicCfg;
use super::ModuleBuilder;

pub struct CrateBuilder {
    pub krate: Crate,
    pub modules: Vec<Box<ModuleBuilder>>,
}

impl CrateBuilder {
    pub fn build(&mut self, cfg: &XicCfg) {
        unimplemented!()
    }

    /// dump is done recursively
    pub fn dump(&self, cfg: &XicCfg) {
        for ctx in self.modules.iter() {
            ctx.dump(cfg);
        }
    }
}

pub fn prepare_crate_for_build(cfg: &XicCfg) -> CrateBuilder {
    let mut mod_path: ItemPathBuf = ItemPathBuf::new();
    mod_path.push(&cfg.crate_name);

    // prepare output dir
    if cfg.out_dir.exists() {
        if !cfg.out_dir.is_dir() {
            panic!(
                "{} already exists but it is not a directory",
                cfg.out_dir.display()
            );
        }
    } else {
        fs::create_dir_all(&cfg.out_dir).unwrap();
    }

    let mut krate_builder = CrateBuilder {
        krate: Crate {
            crate_name: cfg.crate_name.clone(),
            mod_tbl: HashMap::new(),
        },
        modules: Vec::new(),
    };

    super::module::new_module(mod_path, &mut krate_builder, cfg);
    krate_builder
}
