mod class;
mod external;
mod member;
mod module;
mod var;

pub use self::class::Class;
pub use self::member::{Field, Method, Param};
pub use self::module::Module;
pub use self::var::{Locals, Var};
use external::{ExtClass, ExtField, ExtMethod, ExtModule};

use super::super::XicCfg;

use xir::attrib::*;
use xir::util::path::ModPath;

use std::collections::HashMap;
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
    pub fn get_field(&self, name: &str) -> Option<FieldRef> {
        match self {
            ClassRef::Class(c) => {
                if let Some(f) = unsafe { c.as_ref().unwrap().fields.get(name) } {
                    Some(FieldRef::Field(f.as_ref()))
                } else {
                    None
                }
            }
            ClassRef::ExtClass(c) => {
                if let Some(f) = c.fields.get(name) {
                    Some(FieldRef::ExtField(f.as_ref()))
                } else {
                    None
                }
            }
        }
    }

    pub fn get_method(&self, name: &str) -> Option<MethodRef> {
        match self {
            ClassRef::Class(c) => {
                if let Some(m) = unsafe { c.as_ref().unwrap().methods.get(name) } {
                    Some(MethodRef::Method(m.as_ref()))
                } else {
                    None
                }
            }
            ClassRef::ExtClass(c) => {
                if let Some(m) = c.methods.get(name) {
                    Some(MethodRef::ExtMethod(m.as_ref()))
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
                (&c.flag, &c.instance_fields)
            }
            ClassRef::ExtClass(c) => (&c.flag, &c.instance_fields),
        }
    }
}

pub enum FieldRef<'m> {
    Field(&'m Field),
    ExtField(&'m ExtField),
}

impl<'m> FieldRef<'m> {
    pub fn flag(&self) -> FieldAttrib {
        match self {
            FieldRef::Field(f) => f.flag.clone(),
            FieldRef::ExtField(f) => f.flag.clone(),
        }
    }
}

pub enum MethodRef<'m> {
    Method(&'m Method),
    ExtMethod(&'m ExtMethod),
}

impl<'m> MethodRef<'m> {
    pub fn flag(&self) -> MethodAttrib {
        match self {
            MethodRef::Method(m) => m.flag.clone(),
            MethodRef::ExtMethod(m) => m.flag.clone(),
        }
    }
}

pub struct ModMgr {
    pub name: String,

    /// key: mod_fullname
    pub mod_tbl: HashMap<String, Box<ModRef>>,
}

impl ModMgr {
    pub fn new(cfg: &XicCfg) -> ModMgr {
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

        let mut mod_tbl = HashMap::new();
        Module::from_xi(
            mod_path,
            &PathBuf::from(""),
            &cfg.root_path,
            false,
            &mut mod_tbl,
            cfg,
        );

        ModMgr {
            name: cfg.crate_name.to_owned(),
            mod_tbl,
        }
    }

    pub fn build(&mut self, cfg: &XicCfg) {
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
                    m.code_gen(self, cfg);
                }
                ModRef::ExtMod(_) => {}
            }
        }
    }

    /// dump is done recursively
    pub fn dump(&self, cfg: &XicCfg) {
        self.mod_tbl
            .get(&self.name)
            .unwrap()
            .expect_mod()
            .dump(self, &cfg.out_dir);
    }
}
