mod class;
mod external;
mod member;
mod module;
mod var;

pub use self::class::Class;
pub use self::member::{Field, Method, Param};
pub use self::module::Module;
pub use self::var::{Locals, Var};
use external::{ExtClass, ExtModule};

use super::super::XicCfg;

use xir::attrib::*;
use xir::util::path::ModPath;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

pub enum ModRef {
    Mod(Module),
    ExtMod(ExtModule),
}

impl ModRef {
    pub fn expect_mod(&self) -> &Module {
        if let ModRef::Mod(m) = self {
            m
        } else {
            panic!();
        }
    }

    pub fn fullname(&self) -> &str {
        match self {
            ModRef::Mod(m) => m.fullname(),
            ModRef::ExtMod(m) => m.fullname(),
        }
    }

    pub fn get_class(&self, name: &str) -> Option<ClassRef> {
        match self {
            ModRef::Mod(m) => {
                if let Some(c) = m.classes.get(name) {
                    Some(ClassRef::Class(c.as_ptr() as *const Class))
                } else {
                    None
                }
            }
            ModRef::ExtMod(m) => {
                if let Some(c) = m.classes.get(name) {
                    Some(ClassRef::ExtClass(c.as_ref()))
                } else {
                    None
                }
            }
        }
    }

    pub fn contains_sub_mod(&self, mod_name: &str) -> bool {
        match self {
            ModRef::Mod(m) => m.sub_mods.contains(mod_name),
            ModRef::ExtMod(m) => m.sub_mods.contains(mod_name),
        }
    }
}

pub enum ClassRef<'m> {
    // RefCell is bullshit
    Class(*const Class),
    ExtClass(&'m ExtClass),
}

impl<'m> ClassRef<'m> {
    pub fn get_field(&self, name: &str) -> Option<&Field> {
        match self {
            ClassRef::Class(c) => {
                if let Some(f) = unsafe { c.as_ref().unwrap().fields.get(name) } {
                    Some(f.as_ref())
                } else {
                    None
                }
            }
            ClassRef::ExtClass(c) => {
                if let Some(f) = c.fields.get(name) {
                    Some(f.as_ref())
                } else {
                    None
                }
            }
        }
    }

    pub fn get_method(&self, name: &str) -> Option<&Method> {
        match self {
            ClassRef::Class(c) => {
                if let Some(m) = unsafe { c.as_ref().unwrap().methods.get(name) } {
                    Some(m.as_ref())
                } else {
                    None
                }
            }
            ClassRef::ExtClass(c) => {
                if let Some(m) = c.methods.get(name) {
                    Some(m.as_ref())
                } else {
                    None
                }
            }
        }
    }

    pub fn get_info(&self) -> (&TypeAttrib, &Vec<String>) {
        match self {
            ClassRef::Class(c) => {
                let c = unsafe { c.as_ref().unwrap() };
                (&c.attirb, &c.instance_fields)
            }
            ClassRef::ExtClass(c) => (&c.attrib, &c.instance_fields),
        }
    }
}

pub struct ModMgr {
    pub cfg: XicCfg,

    /// key: mod_fullname
    pub mod_tbl: HashMap<String, Box<ModRef>>,
}

impl ModMgr {
    pub fn new(cfg: XicCfg) -> ModMgr {
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

        let cfg = if cfg.crate_name == "std" {
            println!("Info: Compiling stdlib ...");
            cfg
        } else {
            let mut cfg = cfg;
            let mut std_path = env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .to_owned();
            std_path.push("std/std.xibc");
            // TODO: what if std is already present in the ext_paths?
            cfg.ext_paths.push(std_path.canonicalize().unwrap());
            cfg
        };

        let mut mod_tbl = HashMap::new();
        Module::new(
            mod_path,
            &PathBuf::from(""),
            &cfg.root_path,
            false,
            &mut mod_tbl,
            &cfg,
        );

        ModMgr { cfg, mod_tbl }
    }

    pub fn build(&mut self) {
        // 1. member pass
        for gen_mod in self.mod_tbl.values() {
            match gen_mod.as_ref() {
                ModRef::Mod(m) => {
                    m.member_pass(self);
                }
                ModRef::ExtMod(_) => {}
            }
        }

        // 2. code gen
        for gen_mod in self.mod_tbl.values() {
            match gen_mod.as_ref() {
                ModRef::Mod(m) => {
                    m.code_gen(self);
                }
                ModRef::ExtMod(_) => {}
            }
        }
    }

    /// dump is done recursively
    pub fn dump(&self) {
        self.mod_tbl
            .get(&self.cfg.crate_name)
            .unwrap()
            .expect_mod()
            .dump(self, &self.cfg.out_dir);
    }
}
