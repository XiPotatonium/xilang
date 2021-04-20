use super::data::*;
use super::mem::{addr_addu, to_absolute, MemTag, SharedMem, VTblEntry};
use super::native::VMDll;
use super::VMCfg;

use xir::attrib::*;
use xir::blob::IrSig;
use xir::file::*;
use xir::member::{MemberForwarded, MemberRefParent};
use xir::ty::ResolutionScope;
use xir::util::path::{IModPath, ModPath};
use xir::CCTOR_NAME;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::ptr::null;

pub fn load(
    entry: PathBuf,
    mem: &mut SharedMem,
    cfg: &VMCfg,
) -> (Vec<*const Method>, *const Method) {
    let f = IrFile::from_binary(Box::new(fs::File::open(&entry).unwrap()));

    let entrypoint = f.mod_tbl[0].entrypoint as usize;
    if entrypoint == 0 {
        panic!("{} has no entrypoint", entry.display());
    }

    let mut loader = Loader::new(cfg, mem);

    let root = loader.load(f, entry.parent().unwrap());

    (
        loader.cctors,
        mem.mods.get(&root).unwrap().expect_il().methods[entrypoint - 1].as_ref() as *const Method,
    )
}

struct Loader<'c> {
    cfg: &'c VMCfg,
    mem: &'c mut SharedMem,
    str_map: HashMap<String, u32>,
    cctor_name: u32,
    empty_str: u32, // ""
    cctors: Vec<*const Method>,
}

impl<'c> Loader<'c> {
    fn new(cfg: &'c VMCfg, mem: &'c mut SharedMem) -> Loader<'c> {
        let mut loader = Loader {
            mem,
            cfg,
            str_map: HashMap::new(),
            cctor_name: 0,
            empty_str: 0,
            cctors: Vec::new(),
        };
        loader.empty_str = loader.add_const_string(String::from(""));
        loader.cctor_name = loader.add_const_string(String::from(CCTOR_NAME));

        loader
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
        let mut classes: Vec<Box<Type>> = Vec::new();
        let mut methods: Vec<Box<Method>> = Vec::new();
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
            let mut class_methods: Vec<*mut Method> = Vec::new();
            let mut class_fields: Vec<*mut Field> = Vec::new();

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
                        MethodImpl::IL(MethodILImpl {
                            offset: 0,
                            locals: Vec::new(),
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
                        MethodImpl::Native(MethodNativeImpl {
                            scope: impl_map.scope as usize - 1,
                            name: str_heap[impl_map.name as usize],
                            flag: PInvokeAttrib::from(impl_map.flag),
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
                let mut ps: Vec<Param> =
                    if let IrSig::Method(_, ps, _) = &file.blob_heap[method_entry.sig as usize] {
                        (0..ps.len())
                            .into_iter()
                            .map(|_| Param {
                                name: self.empty_str,
                                attrib: ParamAttrib::default(),
                                ty: BuiltinType::Unk,
                            })
                            .collect()
                    } else {
                        panic!();
                    };
                for p in param.iter() {
                    if p.sequence == 0 {
                        // return value
                        continue;
                    }
                    ps[(p.sequence - 1) as usize].name = str_heap[p.name as usize];
                    ps[(p.sequence - 1) as usize].attrib = ParamAttrib::from(p.flag);
                }

                let mut method = Box::new(Method {
                    ctx: null(),
                    name,
                    flag,
                    impl_flag,
                    // fill later
                    parent_class: None,
                    // fill in link stage
                    ret: if let Some(p) = param.first() {
                        if p.sequence == 0 {
                            Param {
                                name: str_heap[p.name as usize],
                                attrib: ParamAttrib::from(p.flag),
                                ty: BuiltinType::Unk,
                            }
                        } else {
                            Param {
                                name: self.empty_str,
                                attrib: ParamAttrib::default(),
                                ty: BuiltinType::Unk,
                            }
                        }
                    } else {
                        Param {
                            name: self.empty_str,
                            attrib: ParamAttrib::default(),
                            ty: BuiltinType::Unk,
                        }
                    },
                    ps,
                    method_impl,
                });

                if name == self.cctor_name {
                    self.cctors.push(method.as_ref() as *const Method);
                }

                class_methods.push(method.as_mut() as *mut Method);
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
                    addr: 0,
                });
                class_fields.push(field.as_mut() as *mut Field);
                fields.push(field);

                field_i += 1;
            }

            let class = Box::new(Type {
                name: class_name,
                attrib: class_flag,
                fields: class_fields,
                methods: class_methods,
                // fill in link stage
                obj_size: 0,
                vtbl_addr: 0,
            });

            for method in class.methods.iter() {
                unsafe {
                    method.as_mut().unwrap().parent_class = Some(class.as_ref() as *const Type)
                };
            }

            classes.push(class);
        }

        let this_mod_fullname_addr = str_heap[file.mod_tbl[0].name as usize];
        let this_mod_path = ModPath::from_str(self.mem.get_str(this_mod_fullname_addr));
        let mut this_mod = Box::new(Module::IL(ILModule {
            classes,

            methods,
            fields,

            // fill in link stage
            memberref: vec![],
            modref: vec![],
            classref: vec![],
        }));
        let this_mod_ptr = this_mod.as_ref() as *const Module;
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
                    Box::new(Module::Native(
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
            .as_mut() as *mut Module;
        unsafe {
            {
                // 3.1 Link modref
                let mod_modref = &mut this_mod.as_mut().unwrap().expect_il_mut().modref;
                for modref in file.modref_tbl.iter() {
                    let name = str_heap[modref.name as usize];
                    mod_modref.push(self.mem.mods.get(&name).unwrap().as_ref() as *const Module);
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
                                mod_typeref.push(class.as_ref() as *const Type);
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

                    match sig {
                        IrSig::Method(_, ps, ret) => {
                            // this member ref is a function
                            let ctx = this_mod.as_ref().unwrap().expect_il();

                            let ret_ty = BuiltinType::from_ir_ele_ty(ret, ctx);

                            match parent_tag {
                                MemberRefParent::TypeRef => {
                                    let parent = mod_typeref[parent_idx];
                                    let ps_ty: Vec<BuiltinType> = ps
                                        .iter()
                                        .map(|p| BuiltinType::from_ir_ele_ty(p, ctx))
                                        .collect();
                                    for m in parent.as_ref().unwrap().methods.iter() {
                                        let m_ref = m.as_ref().unwrap();
                                        if m_ref.name == name
                                            && ret_ty == m_ref.ret.ty
                                            && ps_ty.len() == m_ref.ps.len()
                                        {
                                            let mut is_match = true;
                                            for (p0, p1) in ps_ty.iter().zip(m_ref.ps.iter()) {
                                                if p0 != &p1.ty {
                                                    is_match = false;
                                                    break;
                                                }
                                            }
                                            if is_match {
                                                // method found
                                                mod_memberref.push(MemberRef::Method(*m));
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
                        }
                        IrSig::Field(f_sig) => {
                            // this member ref is a field
                            let sig = BuiltinType::from_ir_ele_ty(
                                f_sig,
                                this_mod.as_ref().unwrap().expect_il(),
                            );
                            match parent_tag {
                                MemberRefParent::TypeRef => {
                                    // check if parent has this field
                                    for f in mod_typeref[parent_idx].as_ref().unwrap().fields.iter()
                                    {
                                        let f_ref = f.as_ref().unwrap();
                                        if f_ref.name == name && sig == f_ref.ty {
                                            // field found
                                            mod_memberref.push(MemberRef::Field(*f));
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
                        IrSig::LocalVar(_) => unreachable!(),
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
                if let IrSig::Field(f_sig) = &file.blob_heap[field_entry.sig as usize] {
                    field.ty =
                        BuiltinType::from_ir_ele_ty(f_sig, this_mod.as_ref().unwrap().expect_il());
                } else {
                    panic!();
                }
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
                let ctx = this_mod.as_ref().unwrap().expect_il();
                if let IrSig::Method(_, ps, ret) = sig {
                    method.ret.ty = BuiltinType::from_ir_ele_ty(ret, ctx);
                    for (i, p) in ps.iter().enumerate() {
                        method.ps[i].ty = BuiltinType::from_ir_ele_ty(p, ctx);
                    }
                } else {
                    panic!();
                }

                match &mut method.method_impl {
                    MethodImpl::IL(il_impl) => {
                        let body = &file.codes[method_entry.body as usize - 1];
                        if body.locals != 0 {
                            if let IrSig::LocalVar(locals) = &file.blob_heap
                                [file.stand_alone_sig_tbl[body.locals as usize - 1].sig as usize]
                            {
                                for var in locals.iter() {
                                    il_impl.locals.push(BuiltinType::from_ir_ele_ty(var, ctx))
                                }
                            }
                        }
                    }
                    MethodImpl::Native(_) => {}
                }
            }

            // 3.6 allocate class static space
            // no alignment
            {
                let this_mod_mut = this_mod.as_mut().unwrap().expect_il_mut();
                for class in this_mod_mut.classes.iter_mut() {
                    let mut instance_field_offset = 0;
                    let mut static_field_offset = 0;

                    for field in class.fields.iter() {
                        let field = field.as_mut().unwrap();

                        // determine field relative offset
                        let field_heap_size = field.ty.heap_size();
                        if field.attrib.is(FieldAttribFlag::Static) {
                            field.addr = static_field_offset + size_of::<VTblEntry>();
                            static_field_offset += field_heap_size;
                        } else {
                            field.addr = instance_field_offset;
                            instance_field_offset += field_heap_size;
                        }
                    }

                    // allocate obj static space
                    class.obj_size = instance_field_offset;
                    let static_addr = to_absolute(
                        MemTag::StaticMem,
                        self.mem.static_area.add_class(
                            VTblEntry {
                                class: class.as_ref() as *const Type,
                                num_virt: 0,
                                num_interface: 0,
                            },
                            vec![],
                            vec![],
                            static_field_offset,
                        ),
                    );
                    class.vtbl_addr = static_addr;
                    // link static field addr
                    for field in class.fields.iter() {
                        let field = field.as_mut().unwrap();
                        if field.attrib.is(FieldAttribFlag::Static) {
                            field.addr = addr_addu(static_addr, field.addr);
                        }
                    }
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
