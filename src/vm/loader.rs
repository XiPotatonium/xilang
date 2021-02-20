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
    if f.entrypoint == 0 {
        panic!("Module {} has no entrypoint", root_name);
    }

    let mut loader = Loader {
        root_name,
        mem,
        cfg,
        static_inits: Vec::new(),
    };

    let root = loader.load(&f);

    (
        loader.static_inits,
        root.methods[f.entrypoint as usize - 1].clone(),
    )
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

        for class in file.class_tbl.iter() {}

        for method in file.method_tbl.iter() {}

        for field in file.field_tbl.iter() {}

        for constant in file.constant_pool.iter() {
            vm_constant.push(match constant {
                Constant::Utf8(s) => VMConstant::Utf8(self.mem.add_const_str(s.to_owned())),
                Constant::String(_) => unimplemented!(),
                Constant::Mod(name) => {
                    if *name != file.mod_name {
                        // external mod
                        ext_mod_set.insert(*name);
                    }
                    VMConstant::None
                }
                _ => VMConstant::None,
            });
        }

        unimplemented!();
    }
}
