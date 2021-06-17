mod linker;

use super::data::*;
use super::exec::internal_calls::register_internal_calls;
use super::native::VMDll;
use super::shared_mem::SharedMem;
use super::util::ptr::NonNull;
use super::VMCfg;

use xir::attrib::*;
use xir::file::*;
use xir::member::MemberForwarded;
use xir::sig::IrSig;
use xir::util::path::{IItemPath, ItemPathBuf};
use xir::CCTOR_NAME;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::ptr;

pub fn load(
    entry: PathBuf,
    mem: &mut SharedMem,
    cfg: &VMCfg,
) -> (Vec<*const MethodDesc>, *const MethodDesc) {
    let f = IrFile::from_binary(Box::new(fs::File::open(&entry).unwrap()));

    let entrypoint = f.mod_tbl[0].entrypoint as usize;
    if entrypoint == 0 {
        panic!("{} has no entrypoint", entry.display());
    }

    let mut loader = Loader::new(cfg, mem);

    // load
    let root = loader.load(f, entry.parent().unwrap());

    // allocate static space for classes
    for module in loader.mem.mods.values_mut() {
        if let Module::IL(module) = module.as_mut() {
            for ty in module.types.iter_mut() {
                linker::calc_type_mem_layout(ty, &loader.mem.str_pool);
            }
        }
    }

    // calc method size info
    for module in loader.mem.mods.values_mut() {
        if let Module::IL(module) = module.as_mut() {
            for ty in module.types.iter_mut() {
                linker::fill_type_method_info(ty);
            }
        }
    }

    // link memberref, we need type info to link member,
    // so member should be linked after method/field info have been calculated
    for module in loader.mem.mods.values_mut() {
        if let Module::IL(module) = module.as_mut() {
            linker::link_memberref(module, &loader.mem.str_pool);
        }
    }

    // register internal calls
    register_internal_calls(loader.mem);

    (
        loader.cctors,
        mem.mods.get(&root).unwrap().expect_il().methods[entrypoint - 1].as_ref()
            as *const MethodDesc,
    )
}

struct Loader<'c> {
    cfg: &'c VMCfg,
    mem: &'c mut SharedMem,
    str_map: HashMap<String, usize>,
    cctor_name: usize,
    cctors: Vec<*const MethodDesc>,
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
        loader.mem.empty_str_idx = loader.add_const_string(String::from(""));
        loader.mem.std_str_idx = loader.add_const_string(String::from("std"));
        loader.cctor_name = loader.add_const_string(String::from(CCTOR_NAME));

        loader
    }

    pub fn add_const_string(&mut self, s: String) -> usize {
        if let Some(ret) = self.str_map.get(&s) {
            *ret
        } else {
            let ret = self.mem.str_pool.len();
            self.str_map.insert(s.clone(), ret);
            self.mem.str_pool.push(s);
            ret
        }
    }

    fn load(&mut self, file: IrFile, root_dir: &Path) -> usize {
        // Some external mods is not loadable modules but dlls
        let mut ext_mods_mask: Vec<bool> = vec![true; file.modref_tbl.len()];
        for implmap in file.implmap_tbl.iter() {
            ext_mods_mask[implmap.scope as usize - 1] = false;
        }

        let str_heap: Vec<usize> = file
            .str_heap
            .iter()
            .map(|s| self.add_const_string(s.clone()))
            .collect();
        let usr_str_heap = file
            .usr_str_heap
            .iter()
            .map(|s| self.add_const_string(s.clone()))
            .collect();
        let this_mod_fullname_addr = str_heap[file.mod_tbl[0].name as usize];

        // 1. Fill classes methods and fields that defined in this file
        let mut types: Vec<Box<Type>> = Vec::new();
        let mut methods: Vec<Box<MethodDesc>> = Vec::new();
        let mut fields: Vec<Box<Field>> = Vec::new();

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

        for (typedef_i, typedef_entry) in file.typedef_tbl.iter().enumerate() {
            let (field_lim, method_lim) = if typedef_i + 1 >= file.typedef_tbl.len() {
                // last type
                (file.field_tbl.len(), file.method_tbl.len())
            } else {
                let next_typedef = &file.typedef_tbl[typedef_i + 1];
                (
                    next_typedef.fields as usize - 1,
                    next_typedef.methods as usize - 1,
                )
            };

            let type_attrib = TypeAttrib::from(typedef_entry.flag);
            let type_name = str_heap[typedef_entry.name as usize];
            let mut type_methods: HashMap<String, NonNull<MethodDesc>> = HashMap::new();
            let mut type_fields: HashMap<usize, NonNull<Field>> = HashMap::new();

            while method_i < method_lim {
                let method_entry = &file.method_tbl[method_i];
                let name = str_heap[method_entry.name as usize];

                let method_attrib = MethodAttrib::from(method_entry.flag);
                let impl_flag = MethodImplAttrib::from(method_entry.impl_flag);

                let method_impl = match impl_flag.code_ty() {
                    MethodImplAttribCodeTypeFlag::IL => {
                        assert_ne!(method_entry.body, 0);
                        // Currently virtual method is not implemented
                        // Callvirt actually call non virtual method, which offset = 0
                        MethodImpl::IL(MethodILImpl {
                            index: method_entry.body as usize - 1,
                            locals: Vec::new(),
                            locals_size: 0,
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
                        MethodImpl::Native(MethodNativeImpl {
                            scope: impl_map.scope as usize - 1,
                            name: str_heap[impl_map.name as usize],
                            flag: PInvokeAttrib::from(impl_map.flag),
                        })
                    }
                    MethodImplAttribCodeTypeFlag::Runtime => {
                        MethodImpl::Runtime(MethodRuntimeImpl {
                            func: Default::default(),
                        })
                    }
                };

                // generate params
                let param = if method_i == file.method_tbl.len() - 1 {
                    // last method
                    &file.param_tbl[(method_entry.param_list as usize - 1)..]
                } else {
                    &file.param_tbl[(method_entry.param_list as usize - 1)
                        ..(file.method_tbl[method_i + 1].param_list as usize - 1)]
                };
                let ps = if let IrSig::Method(_, ps, _) = &file.blob_heap[method_entry.sig as usize]
                {
                    ps
                } else {
                    panic!();
                };
                let method_sig = method_str_desc_from_ir(&file, method_entry.name, ps);
                let mut ps: Vec<Param> = (0..ps.len())
                    .into_iter()
                    .map(|_| Param::new(self.mem.empty_str_idx, ParamAttrib::default()))
                    .collect();
                for p in param.iter() {
                    if p.sequence == 0 {
                        // return value
                        continue;
                    }
                    ps[(p.sequence - 1) as usize].name = str_heap[p.name as usize];
                    ps[(p.sequence - 1) as usize].attrib = ParamAttrib::from(p.flag);
                }

                let mut method = Box::new(MethodDesc {
                    ctx: unsafe { NonNull::new_null() },
                    index: method_i,
                    name,
                    slot: 0,
                    attrib: method_attrib,
                    impl_attrib: impl_flag,
                    // fill later
                    parent: ptr::null(),
                    // fill in link stage
                    ret: if let Some(p) = param.first() {
                        if p.sequence == 0 {
                            Param::new(str_heap[p.name as usize], ParamAttrib::from(p.flag))
                        } else {
                            Param::new(self.mem.empty_str_idx, ParamAttrib::default())
                        }
                    } else {
                        Param::new(self.mem.empty_str_idx, ParamAttrib::default())
                    },
                    ps,
                    ps_size: 0,
                    method_impl,
                });

                if name == self.cctor_name {
                    self.cctors.push(method.as_ref() as *const MethodDesc);
                }

                type_methods.insert(
                    method_sig,
                    NonNull::new(method.as_mut() as *mut MethodDesc).unwrap(),
                );
                methods.push(method);

                method_i += 1;
            }

            while field_i < field_lim {
                let field_entry = &file.field_tbl[field_i];

                let flag = FieldAttrib::from(field_entry.flag);

                let mut field = Box::new(Field {
                    name: str_heap[field_entry.name as usize],
                    attrib: flag,
                    // fill in link stage
                    ty: BuiltinType::Unk,

                    index: field_i,

                    offset: 0,
                    addr: ptr::null_mut(),
                });
                if let Some(_) = type_fields.insert(
                    field.name,
                    NonNull::new(field.as_mut() as *mut Field).unwrap(),
                ) {
                    panic!("Duplicate field name");
                }
                fields.push(field);

                field_i += 1;
            }

            let ty = Box::new(Type::new(
                unsafe { NonNull::new_null() },
                type_name,
                type_attrib,
                type_fields,
                type_methods,
            ));

            for method in ty.ee_class.methods.values() {
                unsafe { method.as_mut().parent = ty.as_ref() as *const Type };
            }

            // link some special types
            if self.mem.str_pool[this_mod_fullname_addr] == "std" {
                if self.mem.str_pool[ty.name] == "String" {
                    self.mem.str_class = ty.as_ref() as *const Type;
                } else if self.mem.str_pool[ty.name] == "Array" {
                    self.mem.arr_class = ty.as_ref() as *const Type;
                }
            }

            types.push(ty);
        }

        let this_mod_path = ItemPathBuf::from_str(&self.mem.str_pool[this_mod_fullname_addr]);
        let mut this_mod = Box::new(Module::IL(ILModule {
            fullname: this_mod_fullname_addr,

            types,

            methods,
            fields,

            usr_str_heap,

            // fill in link stage
            memberref: vec![],
            modrefs: vec![],
            typerefs: vec![],

            ir_file: file,
            str_heap,
        }));
        let this_mod_ptr = NonNull::new(this_mod.as_mut() as *mut Module).unwrap();
        for method in this_mod.expect_il_mut().methods.iter_mut() {
            method.ctx = this_mod_ptr;
        }

        if let Some(_) = self.mem.mods.insert(this_mod_fullname_addr, this_mod) {
            panic!("Duplicated module name");
        }

        let str_heap = &unsafe { this_mod_ptr.as_ref() }.expect_il().str_heap;
        // 2. Recursive load dependencies
        for (ext_mod, mask) in unsafe { this_mod_ptr.as_ref() }
            .expect_il()
            .ir_file
            .modref_tbl
            .iter()
            .zip(ext_mods_mask.into_iter())
        {
            let ext_mod_fullname_addr = str_heap[ext_mod.name as usize];
            if self.mem.mods.contains_key(&ext_mod_fullname_addr) {
                continue;
            }

            let ext_mod_fullname = &self.mem.str_pool[ext_mod_fullname_addr];
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
                    Box::new(Module::Native(
                        VMDll::new_ascii(candidates[0].to_str().unwrap()).unwrap(),
                    )),
                );
            } else {
                let path = ItemPathBuf::from_str(ext_mod_fullname);
                if path.get_root_name().unwrap() == this_mod_path.get_root_name().unwrap() {
                    // module in the same crate
                    let mut sub_mod_path = root_dir.to_owned();
                    for seg_id in path.iter().skip(1) {
                        sub_mod_path.push(seg_id);
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

        // 3. link
        linker::link_modref(this_mod_ptr, &self.mem.mods);
        linker::link_typeref(this_mod_ptr);
        linker::link_type_info(this_mod_ptr);

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
