use super::data::*;
use super::mem::SharedMem;
use super::native::VMDll;
use super::VMCfg;

use xir::attrib::*;
use xir::blob::IrSig;
use xir::file::*;
use xir::member::{MemberForwarded, MemberRefParent};
use xir::ty::{ResolutionScope, TypeDefOrRef};
use xir::util::path::{IModPath, ModPath};
use xir::CCTOR_NAME;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
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

    // load
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
        let mut types: Vec<Box<Type>> = Vec::new();
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

            let type_flag = TypeAttrib::from(typedef_entry.flag);
            let type_name = str_heap[typedef_entry.name as usize];
            let mut type_methods: Vec<*mut Method> = Vec::new();
            let mut type_fields: Vec<*mut Field> = Vec::new();

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
                    attrib: flag,
                    impl_flag,
                    // fill later
                    parent: None,
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

                type_methods.push(method.as_mut() as *mut Method);
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
                type_fields.push(field.as_mut() as *mut Field);
                fields.push(field);

                field_i += 1;
            }

            let ty = Box::new(Type {
                name: type_name,
                attrib: type_flag,
                fields: type_fields,
                methods: type_methods,
                // fill in link stage
                extends: None,
                // fill in allocation stage
                instance_field_size: 0,
                static_field_size: 0,
                vtbl_addr: 0,
            });

            for method in ty.methods.iter() {
                unsafe { method.as_mut().unwrap().parent = Some(ty.as_ref() as *const Type) };
            }

            types.push(ty);
        }

        let this_mod_fullname_addr = str_heap[file.mod_tbl[0].name as usize];
        let this_mod_path = ModPath::from_str(self.mem.get_str(this_mod_fullname_addr));
        let mut this_mod = Box::new(Module::IL(ILModule {
            types,

            methods,
            fields,

            // fill in link stage
            memberref: vec![],
            modrefs: vec![],
            typerefs: vec![],
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
        // 3.1 Link modref
        let this_mod_mut = unsafe { this_mod.as_mut().unwrap().expect_il_mut() };
        for modref in file.modref_tbl.iter() {
            let name = str_heap[modref.name as usize];
            this_mod_mut
                .modrefs
                .push(self.mem.mods.get_mut(&name).unwrap().as_mut() as *mut Module);
        }

        // 3.2 link typeref
        for typeref in file.typeref_tbl.iter() {
            let name = str_heap[typeref.name as usize];
            let (parent_tag, parent_idx) = typeref.get_parent();
            match parent_tag {
                ResolutionScope::Mod => unimplemented!(), // this is ok
                ResolutionScope::ModRef => {
                    let parent = unsafe { this_mod_mut.modrefs[parent_idx].as_mut().unwrap() };
                    let ty = parent
                        .expect_il_mut()
                        .types
                        .iter_mut()
                        .find(|c| c.as_ref().name == name);
                    if let Some(ty) = ty {
                        this_mod_mut.typerefs.push(ty.as_mut() as *mut Type);
                    } else {
                        panic!("External symbol not found");
                    }
                }
                ResolutionScope::TypeRef => unimplemented!(),
            }
        }

        // 3.3 link member ref
        for memberref in file.memberref_tbl.iter() {
            let name = str_heap[memberref.name as usize];
            let mut found = false;

            let sig = &file.blob_heap[memberref.sig as usize];
            let (parent_tag, parent_idx) = memberref.get_parent();
            let parent_idx = parent_idx as usize - 1;

            match sig {
                IrSig::Method(_, ps, ret) => {
                    // this member ref is a function
                    let ret_ty = BuiltinType::from_ir_ele_ty(ret, this_mod_mut);

                    match parent_tag {
                        MemberRefParent::TypeRef => {
                            let parent =
                                unsafe { this_mod_mut.typerefs[parent_idx].as_ref().unwrap() };
                            let ps_ty: Vec<BuiltinType> = ps
                                .iter()
                                .map(|p| BuiltinType::from_ir_ele_ty(p, this_mod_mut))
                                .collect();
                            for m in parent.methods.iter() {
                                let m_ref = unsafe { m.as_ref().unwrap() };
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
                                        this_mod_mut.memberref.push(MemberRef::Method(*m));
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        MemberRefParent::ModRef => {
                            unimplemented!("Member that has no class parent is not implemented");
                        }
                        _ => unreachable!(),
                    }
                }
                IrSig::Field(f_sig) => {
                    // this member ref is a field
                    let sig = BuiltinType::from_ir_ele_ty(f_sig, this_mod_mut);
                    match parent_tag {
                        MemberRefParent::TypeRef => {
                            // check if parent has this field
                            for f in unsafe {
                                this_mod_mut.typerefs[parent_idx]
                                    .as_ref()
                                    .unwrap()
                                    .fields
                                    .iter()
                            } {
                                let f_ref = unsafe { f.as_ref().unwrap() };
                                if f_ref.name == name && sig == f_ref.ty {
                                    // field found
                                    this_mod_mut.memberref.push(MemberRef::Field(*f));
                                    found = true;
                                    break;
                                }
                            }
                        }
                        MemberRefParent::ModRef => {
                            unimplemented!("Member that has no class parent is not implemented");
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

        // to avoid mutable borrow issue
        let this_mod_ref = unsafe { this_mod.as_ref().unwrap().expect_il() };

        // 3.4 fill field type info
        for (field, field_entry) in this_mod_mut.fields.iter_mut().zip(file.field_tbl.iter()) {
            if let IrSig::Field(f_sig) = &file.blob_heap[field_entry.sig as usize] {
                field.ty = BuiltinType::from_ir_ele_ty(f_sig, this_mod_ref);
            } else {
                panic!();
            }
        }

        // 3.5 fill method type info
        for (method, method_entry) in this_mod_mut.methods.iter_mut().zip(file.method_tbl.iter()) {
            let sig = &file.blob_heap[method_entry.sig as usize];
            if let IrSig::Method(_, ps, ret) = sig {
                method.ret.ty = BuiltinType::from_ir_ele_ty(ret, this_mod_ref);
                for (i, p) in ps.iter().enumerate() {
                    method.ps[i].ty = BuiltinType::from_ir_ele_ty(p, this_mod_ref);
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
                                il_impl
                                    .locals
                                    .push(BuiltinType::from_ir_ele_ty(var, this_mod_ref))
                            }
                        }
                    }
                }
                MethodImpl::Native(_) => {}
            }
        }

        // 3.6 Link class extends
        for (ty, type_entry) in this_mod_mut.types.iter_mut().zip(file.typedef_tbl.iter()) {
            if let Some((parent_tag, parent_idx)) = type_entry.get_extends() {
                ty.extends = Some(match parent_tag {
                    TypeDefOrRef::TypeDef => unsafe {
                        this_mod.as_mut().unwrap().expect_il_mut().types[parent_idx].as_mut()
                            as *mut Type
                    },
                    TypeDefOrRef::TypeRef => this_mod_ref.typerefs[parent_idx],
                    TypeDefOrRef::TypeSpec => unimplemented!(),
                });
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
