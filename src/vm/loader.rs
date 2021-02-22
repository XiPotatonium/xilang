use super::data::*;
use super::mem::SharedMem;
use super::VMCfg;

use crate::ir::blob::IrBlob;
use crate::ir::flag::*;
use crate::ir::ir_file::*;
use crate::ir::CCTOR_NAME;

use std::collections::HashMap;
use std::fs;
use std::mem::size_of;
use std::path::PathBuf;
use std::ptr::null;

fn blob_size(blob: &IrBlob) -> usize {
    match blob {
        IrBlob::Void => panic!("Void type has not heap size"),
        IrBlob::Bool => size_of::<i32>(),
        IrBlob::Char => size_of::<u16>(),
        IrBlob::U8 => size_of::<u8>(),
        IrBlob::I8 => size_of::<i8>(),
        IrBlob::U16 => size_of::<u16>(),
        IrBlob::I16 => size_of::<i16>(),
        IrBlob::U32 => size_of::<u32>(),
        IrBlob::I32 => size_of::<i32>(),
        IrBlob::U64 => size_of::<u64>(),
        IrBlob::I64 => size_of::<i64>(),
        IrBlob::UNative => size_of::<usize>(),
        IrBlob::INative => size_of::<isize>(),
        IrBlob::F32 => size_of::<f32>(),
        IrBlob::F64 => size_of::<f64>(),
        IrBlob::Obj(_) => size_of::<usize>(),
        IrBlob::Func(_, _) => unimplemented!("Size of IrBlob::Func is not implemented"),
        IrBlob::Array(_) => size_of::<usize>(),
    }
}

pub fn load(
    entry: PathBuf,
    mem: &mut SharedMem,
    cfg: &VMCfg,
) -> (Vec<*const VMMethod>, *const VMMethod) {
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
        cctor_name: 0,
        static_inits: Vec::new(),
    };
    loader.cctor_name = loader.add_const_str(String::from(CCTOR_NAME));

    let root = loader.load(f);

    (
        loader.static_inits,
        mem.mods.get(&root).unwrap().methods[entrypoint - 1].as_ref() as *const VMMethod,
    )
}

struct Loader<'c> {
    cfg: &'c VMCfg,
    mem: &'c mut SharedMem,
    str_map: HashMap<String, u32>,
    cctor_name: u32,
    static_inits: Vec<*const VMMethod>,
}

impl<'c> Loader<'c> {
    fn to_vm_ty(&self, blob_heap: &Vec<IrBlob>, this_mod: &VMModule, idx: u32) -> VMType {
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
                    TBL_CLASS_TAG => this_mod.classes[idx].as_ref() as *const VMClass,
                    TBL_CLASSREF_TAG => this_mod.classref[idx],
                    _ => panic!(""),
                })
            }
            IrBlob::Func(_, _) => panic!(),
            IrBlob::Array(inner) => self.to_vm_ty(blob_heap, this_mod, *inner),
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
        // println!("{}\n\n\n\n\n\n\n", file);

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
        let mut classes: Vec<Box<VMClass>> = Vec::new();
        let mut methods: Vec<Box<VMMethod>> = Vec::new();
        let mut fields: Vec<Box<VMField>> = Vec::new();
        let mut codes_iter = file.codes.into_iter();

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
                (
                    next_class.fields as usize - 1,
                    next_class.methods as usize - 1,
                )
            };

            let class_flag = TypeFlag::new(class_entry.flag);
            let class_name = str_heap[class_entry.name as usize];
            let mut class_methods: Vec<*const VMMethod> = Vec::new();
            let mut class_fields: Vec<*const VMField> = Vec::new();

            let mut static_field_offset: usize = 0;
            let mut non_static_field_offset: usize = 0;

            while field_i < field_lim {
                let field_entry = &file.field_tbl[field_i];

                let flag = FieldFlag::new(field_entry.flag);
                let field_size = blob_size(&file.blob_heap[field_entry.signature as usize]);
                // TODO alignment
                let offset = if flag.is(FieldFlagTag::Static) {
                    static_field_offset += field_size;
                    static_field_offset - field_size
                } else {
                    non_static_field_offset += field_size;
                    non_static_field_offset - field_size
                };

                let field = Box::new(VMField {
                    name: str_heap[field_entry.name as usize],
                    flag,
                    // fill in link stage
                    ty: VMType::Unk,
                    // link in link stage
                    addr: offset,
                });
                class_fields.push(field.as_ref() as *const VMField);
                fields.push(field);

                field_i += 1;
            }

            while method_i < method_lim {
                let method_entry = &file.method_tbl[method_i];

                let flag = MethodFlag::new(method_entry.flag);
                // Currently virtual method is not implemented
                // Callvirt actually call non virtual method, which offset = 0
                let method = Box::new(VMMethod {
                    ctx: null(),
                    name: str_heap[method_entry.name as usize],
                    flag,
                    // fill in link stage
                    ret_ty: VMType::Unk,
                    ps_ty: Vec::new(),
                    offset: 0,
                    locals: method_entry.locals as usize,
                    insts: codes_iter.next().unwrap(),
                });

                if method.name == self.cctor_name {
                    self.static_inits.push(method.as_ref() as *const VMMethod);
                }

                class_methods.push(method.as_ref() as *const VMMethod);
                methods.push(method);

                method_i += 1;
            }

            classes.push(Box::new(VMClass {
                name: class_name,
                flag: class_flag,
                fields: class_fields,
                methods: class_methods,
                // fill at link stage
                obj_size: 0,
            }));
        }

        let mod_name_addr = str_heap[file.mod_tbl[0].name as usize];
        let mut this_mod = Box::new(VMModule {
            classes,

            methods,
            fields,

            // fill in link stage
            memberref: vec![],
            modref: vec![],
            classref: vec![],
        });
        let this_mod_ptr = this_mod.as_ref() as *const VMModule;
        for method in this_mod.methods.iter_mut() {
            method.ctx = this_mod_ptr;
        }

        if let Some(_) = self.mem.mods.insert(mod_name_addr, this_mod) {
            panic!("Duplicated module name");
        }

        // 2. Recursive load dependencies
        for _ in external_mods.iter() {
            for _ in self.cfg.ext_paths.iter() {
                unimplemented!();
            }
            unimplemented!("Dependency is not implemented");
        }

        // 3. Link extenal symbols

        let this_mod = self.mem.mods.get_mut(&mod_name_addr).unwrap().as_mut() as *mut VMModule;
        unsafe {
            {
                let mod_modref = &mut this_mod.as_mut().unwrap().modref;
                for modref in file.modref_tbl.iter() {
                    let name = str_heap[modref.name as usize];
                    mod_modref.push(self.mem.mods.get(&name).unwrap().as_ref() as *const VMModule);
                }

                let mod_classref = &mut this_mod.as_mut().unwrap().classref;
                for classref in file.classref_tbl.iter() {
                    let name = str_heap[classref.name as usize];
                    let parent_idx = (classref.parent & !TBL_TAG_MASK) as usize - 1;
                    assert_eq!(classref.parent & TBL_TAG_MASK, TBL_MODREF_TAG);
                    let parent = mod_modref[parent_idx].as_ref().unwrap();
                    let class = parent.classes.iter().find(|&c| c.as_ref().name == name);
                    if let Some(class) = class {
                        mod_classref.push(class.as_ref() as *const VMClass);
                    } else {
                        panic!("External symbol not found");
                    }
                }

                let mod_memberref = &mut this_mod.as_mut().unwrap().memberref;
                for memberref in file.memberref_tbl.iter() {
                    let name = str_heap[memberref.name as usize];
                    let parent_idx = (memberref.parent & !TBL_TAG_MASK) as usize - 1;
                    let mut found = false;

                    let sig = &file.blob_heap[memberref.signature as usize];

                    if let IrBlob::Func(ps, ret) = sig {
                        // this member ref is a function
                        let mut ps_ty: Vec<VMType> = Vec::new();
                        for p in ps.iter() {
                            ps_ty.push(self.to_vm_ty(
                                &file.blob_heap,
                                this_mod.as_ref().unwrap(),
                                *p,
                            ));
                        }
                        let ret_ty =
                            self.to_vm_ty(&file.blob_heap, this_mod.as_ref().unwrap(), *ret);

                        match memberref.parent & TBL_TAG_MASK {
                            TBL_CLASSREF_TAG => {
                                for m in mod_classref[parent_idx].as_ref().unwrap().methods.iter() {
                                    let m_ref = m.as_ref().unwrap();
                                    if m_ref.name == name
                                        && ret_ty == m_ref.ret_ty
                                        && ps_ty.len() == m_ref.ps_ty.len()
                                    {
                                        let mut is_match = true;
                                        for (p0, p1) in ps_ty.iter().zip(m_ref.ps_ty.iter()) {
                                            if p0 != p1 {
                                                is_match = false;
                                                break;
                                            }
                                        }
                                        if is_match {
                                            // method found
                                            mod_memberref.push(VMMemberRef::Method(*m));
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            TBL_MODREF_TAG => {
                                unimplemented!();
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        // this member ref is a field
                        unimplemented!();
                    }

                    if !found {
                        panic!("External symbol not found");
                    }
                }
            }

            for (field, field_entry) in this_mod
                .as_mut()
                .unwrap()
                .fields
                .iter_mut()
                .zip(file.field_tbl.iter())
            {
                field.ty = self.to_vm_ty(
                    &file.blob_heap,
                    this_mod.as_ref().unwrap(),
                    field_entry.signature,
                );
            }

            for (method, method_entry) in this_mod
                .as_mut()
                .unwrap()
                .methods
                .iter_mut()
                .zip(file.method_tbl.iter())
            {
                let sig = &file.blob_heap[method_entry.signature as usize];
                if let IrBlob::Func(ps, ret) = sig {
                    method.ret_ty =
                        self.to_vm_ty(&file.blob_heap, this_mod.as_ref().unwrap(), *ret);
                    for p in ps.iter() {
                        method.ps_ty.push(self.to_vm_ty(
                            &file.blob_heap,
                            this_mod.as_ref().unwrap(),
                            *p,
                        ));
                    }
                } else {
                    panic!();
                }
            }
        }

        mod_name_addr
    }
}
