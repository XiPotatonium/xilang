use super::data::*;
use super::mem::SharedMem;
use super::VMCfg;

use crate::ir::ir_file::*;

use std::collections::HashSet;
use std::fs;
use std::rc::Rc;

pub fn load(mem: &mut SharedMem, cfg: &VMCfg) -> (Vec<Rc<VMMethod>>, Rc<VMMethod>) {
    let f = fs::File::open(&cfg.entry).unwrap();
    let f = IrFile::from_binary(Box::new(f));

    let root_name = f.mod_name().unwrap().to_owned();
    let entrypoint = if let Some(m) = f.mod_tbl.first() {
        if m.entrypoint == 0 {
            panic!("{} has no entrypoint", cfg.entry.display());
        } else {
            m.entrypoint as usize
        }
    } else {
        panic!("{} is not a module", cfg.entry.display());
    };

    let mut loader = Loader {
        root_name,
        mem,
        cfg,
        static_inits: Vec::new(),
    };

    let root = loader.load(&f);

    (loader.static_inits, root.methods[entrypoint - 1].clone())
}

struct Loader<'c> {
    root_name: String,
    cfg: &'c VMCfg,
    mem: &'c mut SharedMem,
    static_inits: Vec<Rc<VMMethod>>,
}

impl<'c> Loader<'c> {
    fn load(&mut self, file: &IrFile) -> Rc<VMModule> {
        let mod_name_addr = self.mem.add_const_str(file.mod_name().unwrap().to_owned());

        let mut ext_mod_set: HashSet<u32> = HashSet::new();

        let mut vm_constant: Vec<VMConstant> = vec![VMConstant::None];

        let mut classes: Vec<VMClasse> = Vec::new();
        let mut methods: Vec<VMMethod> = Vec::new();
        let mut fields: Vec<VMField> = Vec::new();

        for ty in file.type_tbl.iter() {}

        for method in file.method_tbl.iter() {}

        for field in file.field_tbl.iter() {}

        unimplemented!();
    }
}
