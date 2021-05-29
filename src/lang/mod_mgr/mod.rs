mod external;
mod member;
mod module;
mod ty;
mod var;

pub use self::member::{Field, Method, Param};
use self::module::new_module;
pub use self::module::{Module, ModuleBuildCtx};
pub use self::ty::Type;
pub use self::var::{Locals, Var};

use super::super::XicCfg;

use xir::util::path::ModPath;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct Crate {
    pub crate_name: String,

    /// key: mod_fullname
    pub mod_tbl: HashMap<String, Box<Module>>,

    pub mod_build_ctx: Vec<ModuleBuildCtx>,
}

impl Crate {
    pub fn new(cfg: &XicCfg) -> Crate {
        let mut mod_path: ModPath = ModPath::new();
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

        let mut mgr = Crate {
            crate_name: cfg.crate_name.clone(),
            mod_tbl: HashMap::new(),
            mod_build_ctx: Vec::new(),
        };

        new_module(
            mod_path,
            &PathBuf::from(""),
            &cfg.root_path,
            false,
            &mut mgr,
            &cfg,
        );
        mgr
    }

    pub fn build(&mut self, cfg: &XicCfg) {
        // 1. member pass
        for ctx in self.mod_build_ctx.iter() {
            ctx.member_pass(self);
        }

        // 2. code gen
        for ctx in self.mod_build_ctx.iter() {
            ctx.code_gen(self, cfg);
        }
    }

    /// dump is done recursively
    pub fn dump(&self) {
        for ctx in self.mod_build_ctx.iter() {
            ctx.dump();
        }
    }
}
