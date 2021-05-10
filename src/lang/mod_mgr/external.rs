///! This file defines external module info
///! Some detail info of module is not loaded
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use xir::attrib::*;
use xir::file::*;
use xir::sig::{self, IrSig, TypeSig};
use xir::tok::{get_tok_tag, TokTag};
use xir::ty::{ResolutionScope, TypeDefOrRef};
use xir::util::path::{IModPath, ModPath};

use super::super::gen::RValType;
use super::{Class, Field, Method, ModRef, Param};

fn to_param(sig: &sig::ParamType, ctx: &IrFile) -> Param {
    Param {
        id: String::from(""),
        attrib: ParamAttrib::default(),
        ty: match &sig.ty {
            sig::InnerParamType::Default(ty) => to_rval(ty, ctx),
            sig::InnerParamType::ByRef(_) => unimplemented!(),
        },
    }
}

fn to_ret(sig: &sig::RetType, ctx: &IrFile) -> RValType {
    match &sig.ty {
        sig::InnerRetType::Default(ty) => to_rval(ty, ctx),
        sig::InnerRetType::ByRef(_) => unimplemented!(),
        sig::InnerRetType::Void => RValType::Void,
    }
}

fn to_rval(sig: &TypeSig, ctx: &IrFile) -> RValType {
    match sig {
        TypeSig::Boolean => RValType::Bool,
        TypeSig::Char => RValType::Char,
        TypeSig::I1 => unimplemented!(),
        TypeSig::U1 => RValType::U8,
        TypeSig::I4 => RValType::I32,
        TypeSig::U4 => unimplemented!(),
        TypeSig::I8 => unimplemented!(),
        TypeSig::U8 => unimplemented!(),
        TypeSig::R4 => unimplemented!(),
        TypeSig::R8 => RValType::F64,
        TypeSig::I => unimplemented!(),
        TypeSig::U => unimplemented!(),
        TypeSig::SZArray(_) => unimplemented!(),
        TypeSig::String => RValType::String,
        TypeSig::Class(tok) => {
            // tok is TypeRef or TypeDef
            let (tag, idx) = get_tok_tag(*tok);
            let idx = idx as usize - 1;
            match tag {
                TokTag::TypeDef => RValType::Obj(
                    ctx.mod_name().to_owned(),
                    ctx.get_str(ctx.typedef_tbl[idx].name).to_owned(),
                ),
                TokTag::TypeRef => {
                    let (parent_tag, parent_idx) = ctx.typeref_tbl[idx].get_parent();
                    match parent_tag {
                        ResolutionScope::Mod => unreachable!(),
                        ResolutionScope::ModRef => RValType::Obj(
                            ctx.get_str(ctx.modref_tbl[parent_idx].name).to_owned(),
                            ctx.get_str(ctx.typeref_tbl[idx].name).to_owned(),
                        ),
                        ResolutionScope::TypeRef => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

pub struct ExtModule {
    pub mod_path: ModPath,
    /// key: mod_name
    pub sub_mods: HashSet<String>,
    /// key: class_name
    pub classes: HashMap<String, Box<Class>>,
}

impl ExtModule {
    pub fn fullname(&self) -> &str {
        self.mod_path.as_str()
    }
}

pub fn load_external_crate(
    mod_tbl: &mut HashMap<String, Box<ModRef>>,
    ext_crate_dir: &Path,
    file: IrFile,
) {
    let mut external_mods_mask: Vec<bool> = vec![true; file.modref_tbl.len()];
    for implmap in file.implmap_tbl.iter() {
        external_mods_mask[implmap.scope as usize - 1] = false;
    }

    let this_mod_path = ModPath::from_str(file.mod_name());
    let mut this_mod = Box::new(ModRef::ExtMod(ExtModule {
        sub_mods: HashSet::new(),
        mod_path: this_mod_path.clone(),
        classes: HashMap::new(),
    }));
    let this_mod_ptr = match this_mod.as_mut() {
        ModRef::Mod(_) => unreachable!(),
        ModRef::ExtMod(m) => m as *mut ExtModule,
    };

    // 1. Fill classes methods and fields that defined in this file
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

        let flag = TypeAttrib::from(class_entry.flag);
        let name = file.get_str(class_entry.name);
        let mut class = Box::new(Class {
            parent: this_mod.as_ref() as *const ModRef,
            name: name.to_owned(),
            methods: HashMap::new(),
            fields: HashMap::new(),
            attrib: flag,
            extends: None,
            // idx of external class will not be used
            idx: 0,
        });

        while method_i < method_lim {
            let method_entry = &file.method_tbl[method_i];
            let param = if method_i == file.method_tbl.len() - 1 {
                // last method
                &file.param_tbl[(method_entry.param_list as usize - 1)..]
            } else {
                &file.param_tbl[(method_entry.param_list as usize - 1)
                    ..(file.method_tbl[method_i + 1].param_list as usize - 1)]
            };

            let flag = MethodAttrib::from(method_entry.flag);
            let impl_flag = MethodImplAttrib::from(method_entry.impl_flag);

            if let IrSig::Method(_, ps, ret) = &file.blob_heap[method_entry.sig as usize] {
                let mut ps: Vec<Param> = ps.iter().map(|t| to_param(t, &file)).collect();
                for p in param.iter() {
                    if p.sequence == 0 {
                        // xilang has no interests about return type
                        continue;
                    }
                    ps[(p.sequence - 1) as usize].id = file.get_str(p.name).to_owned();
                    ps[(p.sequence - 1) as usize].attrib = ParamAttrib::from(p.flag);
                }

                let method = Box::new(Method {
                    parent: class.as_ref() as *const Class,
                    name: file.get_str(method_entry.name).to_owned(),
                    ps,
                    ret: to_ret(ret, &file),
                    attrib: flag,
                    impl_flag,
                    ast: None, // external method has no ast
                    idx: 0,    // idx of external method will not be used
                });

                let method_name = file.get_str(method_entry.name);
                if class.methods.contains_key(method_name) {
                    class.methods.get_mut(method_name).unwrap().push(method);
                } else {
                    class.methods.insert(method_name.to_owned(), vec![method]);
                }
            } else {
                panic!();
            }

            method_i += 1;
        }

        while field_i < field_lim {
            let field_entry = &file.field_tbl[field_i];
            let field_name = file.get_str(field_entry.name);

            let flag = FieldAttrib::from(field_entry.flag);

            if let IrSig::Field(f_sig) = &file.blob_heap[field_entry.sig as usize] {
                let field = Box::new(Field {
                    parent: class.as_ref() as *const Class,
                    name: file.get_str(field_entry.name).to_owned(),
                    attrib: flag,
                    ty: to_rval(f_sig, &file),
                    // idx of external field will not be used
                    idx: 0,
                });
                class.fields.insert(field_name.to_owned(), field);
            } else {
                panic!();
            }

            field_i += 1;
        }

        unsafe {
            this_mod_ptr
                .as_mut()
                .unwrap()
                .classes
                .insert(name.to_owned(), class);
        }
    }

    if let Some(_) = mod_tbl.insert(file.mod_name().to_owned(), this_mod) {
        panic!("Duplicated module name");
    }

    // 2. Recursive load dependencies
    for (external_mod, mask) in file.modref_tbl.iter().zip(external_mods_mask.into_iter()) {
        if mask == false {
            // some external mods is not xir mod, they are dlls
            continue;
        }

        let external_mod_fullname = file.get_str(external_mod.name);
        let path = ModPath::from_str(external_mod_fullname);

        if mod_tbl.contains_key(external_mod_fullname) {
            continue;
        }

        if path.get_root_name().unwrap() == this_mod_path.get_root_name().unwrap() {
            // only load direct external crates
            // external crates of imported crate is not loaded
            let mut sub_mod_path = ext_crate_dir.to_owned();
            for seg in path.iter().skip(1) {
                sub_mod_path.push(seg);
            }
            sub_mod_path.set_extension("xibc");
            let sub_mod_name = path.get_self_name().unwrap();
            unsafe {
                this_mod_ptr
                    .as_mut()
                    .unwrap()
                    .sub_mods
                    .insert(sub_mod_name.to_owned());
            }
            if sub_mod_path.is_file() {
                let sub_mod_file =
                    IrFile::from_binary(Box::new(fs::File::open(&sub_mod_path).unwrap()));
                if sub_mod_file.mod_name() != external_mod_fullname {
                    panic!(
                        "Inconsistent submodule. Expect {} but found {} in submodule {}",
                        external_mod_fullname,
                        sub_mod_file.mod_name(),
                        sub_mod_path.display()
                    );
                }
                load_external_crate(mod_tbl, ext_crate_dir, sub_mod_file);
            } else {
                // try directory
                sub_mod_path.set_file_name(sub_mod_name);
                if sub_mod_path.is_dir() {
                    sub_mod_path.push(format!("{}.xibc", sub_mod_name));
                    if sub_mod_path.is_file() {
                        let sub_mod_file =
                            IrFile::from_binary(Box::new(fs::File::open(&sub_mod_path).unwrap()));
                        if sub_mod_file.mod_name() != external_mod_fullname {
                            panic!(
                                "Inconsistent submodule. Expect {} but found {} in submodule {}",
                                external_mod_fullname,
                                sub_mod_file.mod_name(),
                                sub_mod_path.display()
                            );
                        }
                        load_external_crate(mod_tbl, ext_crate_dir, sub_mod_file);
                    } else {
                        panic!(
                            "Cannot found sub module {}: {} is not file",
                            external_mod_fullname,
                            sub_mod_path.display()
                        );
                    }
                } else {
                    panic!(
                        "Cannot found sub module {}: {} is not dir",
                        external_mod_fullname,
                        sub_mod_path.display()
                    );
                }
            }
        }

        // 3. link extends
        {
            let this_mod_mut = unsafe { this_mod_ptr.as_mut().unwrap() };
            for class_entry in file.typedef_tbl.iter() {
                let mut class_mut = this_mod_mut
                    .classes
                    .get_mut(file.get_str(class_entry.name))
                    .unwrap();
                if let Some((tag, idx)) = class_entry.get_extends() {
                    class_mut.extends = Some(match tag {
                        TypeDefOrRef::TypeDef => mod_tbl
                            .get(file.mod_name())
                            .unwrap()
                            .get_class(file.get_str(file.typedef_tbl[idx].name))
                            .unwrap(),
                        TypeDefOrRef::TypeRef => {
                            let typeref = &file.typeref_tbl[idx];
                            let (parent_tag, parent_idx) = typeref.get_parent();
                            mod_tbl
                                .get(match parent_tag {
                                    ResolutionScope::Mod => file.mod_name(),
                                    ResolutionScope::ModRef => {
                                        file.get_str(file.modref_tbl[parent_idx].name)
                                    }
                                    ResolutionScope::TypeRef => unreachable!(),
                                })
                                .unwrap()
                                .get_class(file.get_str(typeref.name))
                                .unwrap()
                        }
                        TypeDefOrRef::TypeSpec => unimplemented!(),
                    });
                }
            }
        }
    }
}
