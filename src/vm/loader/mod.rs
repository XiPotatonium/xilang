use super::data::*;
use super::mem::{to_absolute, MemTag, SharedMem, VTblEntry};
use super::native::VMDll;
use super::VMCfg;

use xir::attrib::*;
use xir::blob::Blob;
use xir::file::*;
use xir::tok::{get_tok_tag, TokTag};
use xir::util::path::{IModPath, ModPath};
use xir::CCTOR_NAME;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::ptr::null;

fn blob_size(blob: &Blob) -> usize {
    match blob {
        Blob::Void => panic!("Void type has no heap size"),
        Blob::Bool => size_of::<i32>(),
        Blob::Char => size_of::<u16>(),
        Blob::U8 => size_of::<u8>(),
        Blob::I8 => size_of::<i8>(),
        Blob::U16 => size_of::<u16>(),
        Blob::I16 => size_of::<i16>(),
        Blob::U32 => size_of::<u32>(),
        Blob::I32 => size_of::<i32>(),
        Blob::U64 => size_of::<u64>(),
        Blob::I64 => size_of::<i64>(),
        Blob::UNative => size_of::<usize>(),
        Blob::INative => size_of::<isize>(),
        Blob::F32 => size_of::<f32>(),
        Blob::F64 => size_of::<f64>(),
        Blob::Obj(_) => size_of::<usize>(),
        Blob::Func(_, _) => unimplemented!("Size of IrBlob::Func is not implemented"),
        Blob::Array(_) => size_of::<usize>(),
    }
}

pub fn load(
    entry: PathBuf,
    mem: &mut SharedMem,
    cfg: &VMCfg,
) -> (Vec<*const VMMethod>, *const VMMethod) {
    let f = IrFile::from_binary(Box::new(fs::File::open(&entry).unwrap()));

    let entrypoint = f.mod_tbl[0].entrypoint as usize;
    if entrypoint == 0 {
        panic!("{} has no entrypoint", entry.display());
    }

    let mut loader = Loader::new(cfg, mem);

    let root = loader.load(f, entry.parent().unwrap());

    (
        loader.cctors,
        mem.mods.get(&root).unwrap().expect_il().methods[entrypoint - 1].as_ref()
            as *const VMMethod,
    )
}

struct Loader<'c> {
    cfg: &'c VMCfg,
    mem: &'c mut SharedMem,
    str_map: HashMap<String, u32>,
    cctor_name: u32,
    cctors: Vec<*const VMMethod>,
}

impl<'c> Loader<'c> {
    fn new(cfg: &'c VMCfg, mem: &'c mut SharedMem) -> Loader<'c> {
        let mut loader = Loader {
            mem,
            cfg,
            str_map: HashMap::new(),
            cctor_name: 0,
            cctors: Vec::new(),
        };
        loader.cctor_name = loader.add_const_string(String::from(CCTOR_NAME));

        loader
    }

    fn to_vm_ty(&self, blob_heap: &Vec<Blob>, this_mod: &VMILModule, idx: u32) -> VMBuiltinType {
        let sig = &blob_heap[idx as usize];

        match sig {
            Blob::Void => VMBuiltinType::Void,
            Blob::Bool => VMBuiltinType::Bool,
            Blob::Char => VMBuiltinType::Char,
            Blob::U8 => VMBuiltinType::U8,
            Blob::I8 => VMBuiltinType::I8,
            Blob::U16 => VMBuiltinType::U16,
            Blob::I16 => VMBuiltinType::I16,
            Blob::U32 => VMBuiltinType::U32,
            Blob::I32 => VMBuiltinType::I32,
            Blob::U64 => VMBuiltinType::U64,
            Blob::I64 => VMBuiltinType::I64,
            Blob::UNative => VMBuiltinType::UNative,
            Blob::INative => VMBuiltinType::INative,
            Blob::F32 => VMBuiltinType::F32,
            Blob::F64 => VMBuiltinType::F64,
            Blob::Obj(tok) => {
                let (tag, idx) = get_tok_tag(*tok);
                let idx = idx as usize - 1;
                VMBuiltinType::Obj(match tag {
                    TokTag::TypeDef => this_mod.classes[idx].as_ref() as *const VMType,
                    TokTag::TypeRef => this_mod.classref[idx],
                    _ => unreachable!(),
                })
            }
            Blob::Func(_, _) => panic!(),
            Blob::Array(inner) => self.to_vm_ty(blob_heap, this_mod, *inner),
        }
    }

    pub fn add_const_string(&mut self, s: String) -> u32 {
        if let Some(ret) = self.str_map.get(&s) {
            *ret
        } else {
            let ret = self.mem.str_pool.len() as u32;
            self.str_map.insert(s.clone(), ret);
            self.mem.str_pool.push(s);
            ret
        }
    }

    fn load(&mut self, file: IrFile, root_dir: &Path) -> u32 {
        // Some external mods is not loadable modules but dlls
        let mut ext_mods_mask: Vec<bool> = vec![true; file.modref_tbl.len()];
        for implmap in file.implmap_tbl.iter() {
            ext_mods_mask[implmap.scope as usize - 1] = false;
        }

        let str_heap: Vec<u32> = file
            .str_heap
            .into_iter()
            .map(|s| self.add_const_string(s))
            .collect();

        // 1. Fill classes methods and fields that defined in this file
        let mut classes: Vec<Box<VMType>> = Vec::new();
        let mut methods: Vec<Box<VMMethod>> = Vec::new();
        let mut fields: Vec<Box<VMField>> = Vec::new();

        let (mut field_i, mut method_i) = if let Some(c0) = file.typedef_tbl.first() {
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

        for (class_i, class_entry) in file.typedef_tbl.iter().enumerate() {
            let (field_lim, method_lim) = if class_i + 1 >= file.typedef_tbl.len() {
                // last class
                (file.field_tbl.len(), file.method_tbl.len())
            } else {
                let next_class = &file.typedef_tbl[class_i + 1];
                (
                    next_class.fields as usize - 1,
                    next_class.methods as usize - 1,
                )
            };

            let class_flag = TypeAttrib::from(class_entry.flag);
            let class_name = str_heap[class_entry.name as usize];
            let mut class_methods: Vec<*const VMMethod> = Vec::new();
            let mut class_fields: Vec<*const VMField> = Vec::new();

            let mut static_field_offset: usize = 0;
            let mut non_static_field_offset: usize = 0;
            let vtbl_size = size_of::<VTblEntry>();

            while method_i < method_lim {
                let method_entry = &file.method_tbl[method_i];
                let name = str_heap[method_entry.name as usize];

                let flag = MethodAttrib::from(method_entry.flag);
                let impl_flag = MethodImplAttrib::from(method_entry.impl_flag);

                let method_impl = match impl_flag.code_ty() {
                    MethodImplAttribCodeTypeFlag::IL => {
                        assert_ne!(method_entry.body, 0);
                        let body = &file.codes[method_entry.body as usize - 1];
                        // Currently virtual method is not implemented
                        // Callvirt actually call non virtual method, which offset = 0
                        VMMethodImpl::IL(VMMethodILImpl {
                            offset: 0,
                            locals: body.local as usize,
                            insts: body.insts.to_owned(),
                        })
                    }
                    MethodImplAttribCodeTypeFlag::Native => {
                        // O(N), might need optimization
                        let impl_map = file
                            .implmap_tbl
                            .iter()
                            .find(|&i| {
                                let (member_tag, member_idx) = i.get_member();
                                assert_eq!(member_tag, MemberForwarded::MethodDef);
                                member_idx as usize - 1 == method_i
                            })
                            .unwrap();
                        VMMethodImpl::Native(VMMethodNativeImpl {
                            scope: impl_map.scope as usize - 1,
                            name: str_heap[impl_map.name as usize],
                            flag: PInvokeAttrib::from(impl_map.flag),
                        })
                    }
                };

                let method = Box::new(VMMethod {
                    ctx: null(),
                    name,
                    flag,
                    impl_flag,
                    // fill in link stage
                    ret_ty: VMBuiltinType::Unk,
                    ps_ty: Vec::new(),
                    method_impl,
                });

                if name == self.cctor_name {
                    self.cctors.push(method.as_ref() as *const VMMethod);
                }

                class_methods.push(method.as_ref() as *const VMMethod);
                methods.push(method);

                method_i += 1;
            }

            while field_i < field_lim {
                let field_entry = &file.field_tbl[field_i];

                let flag = FieldAttrib::from(field_entry.flag);
                let field_size = blob_size(&file.blob_heap[field_entry.sig as usize]);
                // TODO alignment
                let offset = if flag.is(FieldAttribFlag::Static) {
                    static_field_offset += field_size;
                    static_field_offset
                } else {
                    non_static_field_offset += field_size;
                    non_static_field_offset
                } - field_size;

                let field = Box::new(VMField {
                    name: str_heap[field_entry.name as usize],
                    attrib: flag,
                    // fill in link stage
                    ty: VMBuiltinType::Unk,
                    addr: offset,
                });
                class_fields.push(field.as_ref() as *const VMField);
                fields.push(field);

                field_i += 1;
            }

            let mut class = Box::new(VMType {
                name: class_name,
                attrib: class_flag,
                fields: class_fields,
                methods: class_methods,
                obj_size: non_static_field_offset,
                vtbl_addr: 0,
            });

            // prepare class static space
            let addr = unsafe {
                to_absolute(
                    MemTag::StaticMem,
                    self.mem.static_area.add_class(
                        VTblEntry {
                            class: class.as_ref() as *const VMType,
                            num_virt: 0,
                            num_interface: 0,
                        },
                        vec![],
                        vec![],
                        static_field_offset,
                    ),
                )
            };
            class.vtbl_addr = addr;

            let fields_len = fields.len();
            for i in (0..class.fields.len())
                .into_iter()
                .map(|i| fields_len - i - 1)
            {
                if fields[i].attrib.is(FieldAttribFlag::Static) {
                    fields[i].addr += addr + vtbl_size;
                }
            }

            classes.push(class);
        }

        let this_mod_fullname_addr = str_heap[file.mod_tbl[0].name as usize];
        let this_mod_path = ModPath::from_str(self.mem.get_str(this_mod_fullname_addr));
        let mut this_mod = Box::new(VMModule::IL(VMILModule {
            classes,

            methods,
            fields,

            // fill in link stage
            memberref: vec![],
            modref: vec![],
            classref: vec![],
        }));
        let this_mod_ptr = this_mod.as_ref() as *const VMModule;
        for method in this_mod.expect_il_mut().methods.iter_mut() {
            method.ctx = this_mod_ptr;
        }

        if let Some(_) = self.mem.mods.insert(this_mod_fullname_addr, this_mod) {
            panic!("Duplicated module name");
        }

        // 2. Recursive load dependencies
        for (ext_mod, mask) in file.modref_tbl.iter().zip(ext_mods_mask.into_iter()) {
            let ext_mod_fullname_addr = str_heap[ext_mod.name as usize];
            if self.mem.mods.contains_key(&ext_mod_fullname_addr) {
                continue;
            }

            let ext_mod_fullname = self.mem.get_str(ext_mod_fullname_addr);
            if mask == false {
                // some external mods is not xir mod, they are dlls
                let candidates = self.find_mod(&ext_mod_fullname);
                if candidates.len() == 0 {
                    panic!("Cannot find external mod {}", ext_mod_fullname);
                } else if candidates.len() != 1 {
                    panic!(
                        "Ambiguous external mod {}:\n{}",
                        ext_mod_fullname,
                        candidates
                            .iter()
                            .map(|p| p.to_str().unwrap())
                            .collect::<Vec<&str>>()
                            .join("\n")
                    );
                }
                self.mem.mods.insert(
                    ext_mod_fullname_addr,
                    Box::new(VMModule::Native(
                        VMDll::new_ascii(candidates[0].to_str().unwrap()).unwrap(),
                    )),
                );
            } else {
                let path = ModPath::from_str(ext_mod_fullname);
                if path.get_root_name().unwrap() == this_mod_path.get_root_name().unwrap() {
                    // module in the same crate
                    let mut sub_mod_path = root_dir.to_owned();
                    for seg in path.iter().skip(1) {
                        sub_mod_path.push(seg);
                    }

                    sub_mod_path.set_extension("xibc");
                    if sub_mod_path.is_file() {
                        let sub_mod_file =
                            IrFile::from_binary(Box::new(fs::File::open(&sub_mod_path).unwrap()));
                        if sub_mod_file.mod_name() != ext_mod_fullname {
                            panic!(
                                "Inconsistent submodule. Expect {} but found {} in submodule {}",
                                ext_mod_fullname,
                                sub_mod_file.mod_name(),
                                sub_mod_path.display()
                            );
                        }
                        self.load(sub_mod_file, root_dir);
                    } else {
                        panic!(
                            "Cannot find module {}: {} is not file",
                            path,
                            sub_mod_path.display()
                        );
                    }
                } else {
                    // external module
                    let candidates = self.find_mod(&format!("{}.xibc", ext_mod_fullname));
                    if candidates.len() == 0 {
                        panic!("Cannot find external mod {}", ext_mod_fullname);
                    } else if candidates.len() != 1 {
                        panic!(
                            "Ambiguous external mod {}:\n{}",
                            ext_mod_fullname,
                            candidates
                                .iter()
                                .map(|p| p.to_str().unwrap())
                                .collect::<Vec<&str>>()
                                .join("\n")
                        );
                    }
                    let sub_mod_file =
                        IrFile::from_binary(Box::new(fs::File::open(&candidates[0]).unwrap()));
                    if sub_mod_file.mod_name() != ext_mod_fullname {
                        panic!(
                            "Inconsistent submodule. Expect {} but found {} in submodule {}",
                            ext_mod_fullname,
                            sub_mod_file.mod_name(),
                            candidates[0].display()
                        );
                    }
                    self.load(sub_mod_file, candidates[0].parent().unwrap());
                }
            }
        }

        // 3. Link extenal symbols
        let this_mod = self
            .mem
            .mods
            .get_mut(&this_mod_fullname_addr)
            .unwrap()
            .as_mut() as *mut VMModule;
        unsafe {
            {
                // 3.1 Link modref
                let mod_modref = &mut this_mod.as_mut().unwrap().expect_il_mut().modref;
                for modref in file.modref_tbl.iter() {
                    let name = str_heap[modref.name as usize];
                    mod_modref.push(self.mem.mods.get(&name).unwrap().as_ref() as *const VMModule);
                }

                // 3.2 link typeref
                let mod_typeref = &mut this_mod.as_mut().unwrap().expect_il_mut().classref;
                for typeref in file.typeref_tbl.iter() {
                    let name = str_heap[typeref.name as usize];
                    let (parent_tag, parent_idx) = typeref.get_parent();
                    match parent_tag {
                        ResolutionScope::Mod => unimplemented!(), // this is ok
                        ResolutionScope::ModRef => {
                            let parent = mod_modref[parent_idx as usize - 1].as_ref().unwrap();
                            let class = parent
                                .expect_il()
                                .classes
                                .iter()
                                .find(|&c| c.as_ref().name == name);
                            if let Some(class) = class {
                                mod_typeref.push(class.as_ref() as *const VMType);
                            } else {
                                panic!("External symbol not found");
                            }
                        }
                        ResolutionScope::TypeRef => unimplemented!(),
                    }
                }

                // 3.3 link member ref
                let mod_memberref = &mut this_mod.as_mut().unwrap().expect_il_mut().memberref;
                for memberref in file.memberref_tbl.iter() {
                    let name = str_heap[memberref.name as usize];
                    let mut found = false;

                    let sig = &file.blob_heap[memberref.sig as usize];
                    let (parent_tag, parent_idx) = memberref.get_parent();
                    let parent_idx = parent_idx as usize - 1;

                    if let Blob::Func(ps, ret) = sig {
                        // this member ref is a function
                        let mut ps_ty: Vec<VMBuiltinType> = Vec::new();
                        for p in ps.iter() {
                            ps_ty.push(self.to_vm_ty(
                                &file.blob_heap,
                                this_mod.as_ref().unwrap().expect_il(),
                                *p,
                            ));
                        }
                        let ret_ty = self.to_vm_ty(
                            &file.blob_heap,
                            this_mod.as_ref().unwrap().expect_il(),
                            *ret,
                        );

                        match parent_tag {
                            MemberRefParent::TypeRef => {
                                for m in mod_typeref[parent_idx].as_ref().unwrap().methods.iter() {
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
                            MemberRefParent::ModRef => {
                                unimplemented!(
                                    "Member that has no class parent is not implemented"
                                );
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        // this member ref is a field
                        let sig = self.to_vm_ty(
                            &file.blob_heap,
                            this_mod.as_ref().unwrap().expect_il(),
                            memberref.sig,
                        );
                        match parent_tag {
                            MemberRefParent::TypeRef => {
                                for f in mod_typeref[parent_idx].as_ref().unwrap().fields.iter() {
                                    let f_ref = f.as_ref().unwrap();
                                    if f_ref.name == name && sig == f_ref.ty {
                                        // field found
                                        mod_memberref.push(VMMemberRef::Field(*f));
                                        found = true;
                                        break;
                                    }
                                }
                            }
                            MemberRefParent::ModRef => {
                                unimplemented!(
                                    "Member that has no class parent is not implemented"
                                );
                            }
                            _ => unreachable!(),
                        }
                    }

                    if !found {
                        panic!("External symbol not found");
                    }
                }
            }

            // 3.4 fill field type info
            for (field, field_entry) in this_mod
                .as_mut()
                .unwrap()
                .expect_il_mut()
                .fields
                .iter_mut()
                .zip(file.field_tbl.iter())
            {
                field.ty = self.to_vm_ty(
                    &file.blob_heap,
                    this_mod.as_ref().unwrap().expect_il(),
                    field_entry.sig,
                );
            }

            // 3.5 fill method type info
            for (method, method_entry) in this_mod
                .as_mut()
                .unwrap()
                .expect_il_mut()
                .methods
                .iter_mut()
                .zip(file.method_tbl.iter())
            {
                let sig = &file.blob_heap[method_entry.sig as usize];
                if let Blob::Func(ps, ret) = sig {
                    method.ret_ty = self.to_vm_ty(
                        &file.blob_heap,
                        this_mod.as_ref().unwrap().expect_il(),
                        *ret,
                    );
                    for p in ps.iter() {
                        method.ps_ty.push(self.to_vm_ty(
                            &file.blob_heap,
                            this_mod.as_ref().unwrap().expect_il(),
                            *p,
                        ));
                    }
                } else {
                    panic!();
                }
            }
        }

        this_mod_fullname_addr
    }

    /// fname is file name of mod. If mod is named "Foo", then fname is something like "Foo.xibc"
    fn find_mod<S: AsRef<OsStr>>(&self, fname: &S) -> Vec<PathBuf> {
        let os_fname = OsStr::new(fname);
        let mut candidates: Vec<PathBuf> = Vec::new();
        for ext in self.cfg.ext_paths.iter() {
            if ext.is_dir() {
                let candidate_path = ext.join(os_fname);
                if candidate_path.is_file() {
                    // hit
                    candidates.push(candidate_path);
                }
            } else if ext.is_file() {
                if ext.file_name().unwrap() == os_fname {
                    // hit
                    candidates.push(ext.clone());
                }
            }
        }
        candidates
    }
}
