///! This file defines external module info
///! Some detail info of module is not loaded
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use xir::attrib::*;
use xir::blob::IrSig;
use xir::file::*;
use xir::util::path::{IModPath, ModPath};

use super::super::gen::RValType;
use super::{Field, Method, ModRef, Param};

pub struct ExtModule {
    pub mod_path: ModPath,
    /// key: mod_name
    pub sub_mods: HashSet<String>,
    /// key: class_name
    pub classes: HashMap<String, Box<ExtClass>>,
}

impl ExtModule {
    pub fn fullname(&self) -> &str {
        self.mod_path.as_str()
    }
}

pub struct ExtClass {
    pub name: String,

    // TODO: delete non_static_fields, we don't need this optimization, iterate over fields is fast enough
    /// Used in new expr
    pub instance_fields: Vec<String>,
    /// key: field_name
    pub fields: HashMap<String, Box<Field>>,
    /// Overload is currently not supported
    ///
    /// key: method_name
    pub methods: HashMap<String, Box<Method>>,

    pub attrib: TypeAttrib,
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

    // 1. Fill classes methods and fields that defined in this file
    let mut classes = HashMap::new();

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

        let mut methods = HashMap::new();
        let mut fields = HashMap::new();
        let mut instance_fields = Vec::new();

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
                let mut ps: Vec<Param> = ps
                    .iter()
                    .map(|t| Param {
                        id: String::from(""),
                        attrib: ParamAttrib::default(),
                        ty: RValType::from_ir_ele_ty(t, &file),
                    })
                    .collect();
                for p in param.iter() {
                    if p.sequence == 0 {
                        // xilang has no interests about return type
                        continue;
                    }
                    ps[(p.sequence - 1) as usize].id = file.get_str(p.name).to_owned();
                    ps[(p.sequence - 1) as usize].attrib = ParamAttrib::from(p.flag);
                }

                let method = Box::new(Method {
                    ps,
                    ret: RValType::from_ir_ele_ty(ret, &file),
                    attrib: flag,
                    impl_flag,
                    idx: method_i as u32 + 1,
                });
                methods.insert(file.get_str(method_entry.name).to_owned(), method);
            } else {
                panic!();
            }

            method_i += 1;
        }

        while field_i < field_lim {
            let field_entry = &file.field_tbl[field_i];
            let field_name = file.get_str(field_entry.name);

            let flag = FieldAttrib::from(field_entry.flag);

            if !flag.is(FieldAttribFlag::Static) {
                instance_fields.push(field_name.to_owned());
            }

            if let IrSig::Field(f_sig) = &file.blob_heap[field_entry.sig as usize] {
                let field = Box::new(Field {
                    attrib: flag,
                    ty: RValType::from_ir_ele_ty(f_sig, &file),
                    idx: field_i as u32 + 1,
                });
                fields.insert(field_name.to_owned(), field);
            } else {
                panic!();
            }

            field_i += 1;
        }

        let flag = TypeAttrib::from(class_entry.flag);
        let name = file.get_str(class_entry.name);
        let class = Box::new(ExtClass {
            name: name.to_owned(),
            methods,
            fields,
            attrib: flag,
            instance_fields,
        });

        classes.insert(name.to_owned(), class);
    }

    let this_mod_path = ModPath::from_str(file.mod_name());
    // 2. Recursive load dependencies
    let mut sub_mods = HashSet::new();
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
            sub_mods.insert(sub_mod_name.to_owned());
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
    }

    let this_mod = ExtModule {
        sub_mods,
        mod_path: this_mod_path,
        classes,
    };
    if let Some(_) = mod_tbl.insert(
        file.mod_name().to_owned(),
        Box::new(ModRef::ExtMod(this_mod)),
    ) {
        panic!("Duplicated module name");
    }
}
