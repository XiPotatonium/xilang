use super::data::*;
use super::mem::SharedMem;
use super::VMCfg;

use crate::ir::flag::*;
use crate::ir::ir_file::*;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub fn load(entry: PathBuf, mem: &mut SharedMem, cfg: &VMCfg) -> (Vec<Rc<VMMethod>>, Rc<VMMethod>) {
    let f = fs::File::open(&entry).unwrap();
    let f = IrFile::from_binary(Box::new(f));

    let entrypoint = (f.mod_tbl[0].entrypoint & !TBL_TAG_MASK) as usize;
    if entrypoint == 0 {
        panic!("{} has no entrypoint", entry.display());
    }

    let mut loader = Loader {
        mem,
        cfg,
        str_map: HashMap::new(),
        static_inits: Vec::new(),
    };

    let root = loader.load(f);

    (
        loader.static_inits,
        mem.mods.get(&root).unwrap().methods[entrypoint - 1].clone(),
    )
}

struct Loader<'c> {
    cfg: &'c VMCfg,
    mem: &'c mut SharedMem,
    str_map: HashMap<String, u32>,
    static_inits: Vec<Rc<VMMethod>>,
}

impl<'c> Loader<'c> {
    fn to_vm_ty(
        &self,
        blob_heap: &Vec<IrBlob>,
        class_tbl: &Vec<Rc<VMClass>>,
        classref_tbl: &Vec<Rc<VMClass>>,
        idx: u32,
    ) -> VMType {
        let sig = &blob_heap[idx as usize];

        match sig {
            IrBlob::Void => VMType::Void,
            IrBlob::Bool => VMType::Bool,
            IrBlob::Char => VMType::Char,
            IrBlob::U8 => VMType::U8,
            IrBlob::I8 => VMType::I8,
            IrBlob::U16 => VMType::U16,
            IrBlob::I16 => VMType::I16,
            IrBlob::U32 => VMType::U32,
            IrBlob::I32 => VMType::I32,
            IrBlob::U64 => VMType::U64,
            IrBlob::I64 => VMType::I64,
            IrBlob::UNative => VMType::UNative,
            IrBlob::INative => VMType::INative,
            IrBlob::F32 => VMType::F32,
            IrBlob::F64 => VMType::F64,
            IrBlob::Obj(idx) => {
                let tag = *idx & TBL_TAG_MASK;
                let idx = (*idx & !TBL_TAG_MASK) as usize;
                let idx = if idx == 0 {
                    panic!("");
                } else {
                    idx - 1
                };

                VMType::Obj(match tag {
                    TBL_CLASS_TAG => class_tbl[idx].clone(),
                    TBL_CLASSREF_TAG => classref_tbl[idx].clone(),
                    _ => panic!(""),
                })
            }
            IrBlob::Func(_, _) => panic!(),
            IrBlob::Array(inner) => self.to_vm_ty(blob_heap, class_tbl, classref_tbl, *inner),
        }
    }

    pub fn add_const_str(&mut self, s: String) -> u32 {
        if let Some(ret) = self.str_map.get(&s) {
            *ret
        } else {
            let ret = self.mem.str_pool.len() as u32;
            self.str_map.insert(s.clone(), ret);
            self.mem.str_pool.push(s);
            ret
        }
    }

    fn load(&mut self, file: IrFile) -> u32 {
        let external_mods: Vec<String> = file
            .modref_tbl
            .iter()
            .map(|entry| file.str_heap[entry.name as usize].clone())
            .collect();

        let str_heap: Vec<u32> = file
            .str_heap
            .into_iter()
            .map(|s| self.add_const_str(s))
            .collect();

        // 1. Fill classes methods and fields that defined in this file
        let mut class_map: HashMap<u32, usize> = HashMap::new();
        let mut classes: Vec<Rc<VMClass>> = Vec::new();
        let mut methods: Vec<Rc<VMMethod>> = Vec::new();
        let mut fields: Vec<Rc<VMField>> = Vec::new();

        let (mut field_i, mut method_i) = if let Some(c0) = file.class_tbl.first() {
            (c0.fields as usize - 1, c0.methods as usize - 1)
        } else {
            (file.field_tbl.len(), file.method_tbl.len())
        };

        for _ in (0..field_i).into_iter() {
            // load field
            unimplemented!("Load field that has no class parent is not implemented");
        }

        for _ in (0..method_i).into_iter() {
            // load methods
            unimplemented!("Load method that has no class parent is not implemented");
        }

        for (class_i, class_entry) in file.class_tbl.iter().enumerate() {
            let (field_lim, method_lim) = if class_i + 1 >= file.class_tbl.len() {
                // last class
                (file.field_tbl.len(), file.method_tbl.len())
            } else {
                let next_class = &file.class_tbl[class_i + 1];
                (next_class.fields as usize, next_class.methods as usize)
            };

            let class_flag = TypeFlag::new(class_entry.flag);
            let class_name = str_heap[class_entry.name as usize];

            while field_i < field_lim {
                let field_entry = &file.field_tbl[field_i];
                let offset = 0;

                fields.push(Rc::new(VMField {
                    name: str_heap[field_entry.name as usize],
                    flag: FieldFlag::new(field_entry.flag),
                    // fill in link stage
                    ty: VMType::Unk,
                    offset,
                }));

                field_i += 1;
            }

            while method_i < method_lim {
                let method_entry = &file.method_tbl[method_i];

                let flag = MethodFlag::new(method_entry.flag);
                let ret = Rc::new(VMMethod {
                    name: str_heap[method_entry.name as usize],
                    flag,
                    // fill in link stage
                    ret_ty: VMType::Unk,
                    ps_ty: Vec::new(),
                    offset: 0,
                    insts: Vec::new(),
                });

                if ret.flag.is(MethodFlagTag::Static) {
                    self.static_inits.push(ret.clone());
                }

                methods.push(ret);

                method_i += 1;
            }

            class_map.insert(class_name, classes.len());
            classes.push(Rc::new(VMClass {
                name: class_name,
                flag: class_flag,
            }));
        }

        let mod_name_addr = str_heap[file.mod_tbl[0].name as usize];

        if let Some(_) = self.mem.mods.insert(
            mod_name_addr,
            Box::new(VMModule {
                class_map,
                classes,
                methods,
                fields,
            }),
        ) {
            panic!();
        }

        // 2. Recursive load dependencies
        for _ in external_mods.iter() {
            for _ in self.cfg.ext_paths.iter() {
                unimplemented!();
            }
            unimplemented!("Dependency is not implemented");
        }

        // 3. Link extenal symbols
        let mut classref_tbl: Vec<Rc<VMClass>> = Vec::new();
        for classref in file.classref_tbl.iter() {
            let class_name = str_heap[classref.name as usize];
            let parent = self.mem.mods.get(&classref.parent).unwrap();
            let class_idx = parent.class_map.get(&class_name).unwrap();
            classref_tbl.push(parent.classes[*class_idx].clone());
        }

        let this_mod = self.mem.mods.get_mut(&mod_name_addr).unwrap().as_mut() as *mut VMModule;
        unsafe {
            let this_mod_class_tbl = &this_mod.as_ref().unwrap().classes;
            for (field, field_entry) in this_mod
                .as_mut()
                .unwrap()
                .fields
                .iter_mut()
                .zip(file.field_tbl.iter())
            {
                let field = Rc::get_mut(field).unwrap();
                field.ty = self.to_vm_ty(
                    &file.blob_heap,
                    this_mod_class_tbl,
                    &classref_tbl,
                    field_entry.signature,
                );
            }
        }

        mod_name_addr
    }
}
