use xir::blob::IrSig;
use xir::file::IrFile;
use xir::member::MemberRefParent;
use xir::ty::{ResolutionScope, TypeDefOrRef};

use super::super::data::{method_sig, BuiltinType, MemberRef, MethodImpl, Module, Type};

use std::collections::HashMap;

pub fn link_modref(
    file: &IrFile,
    this_mod: *mut Module,
    str_heap: &Vec<usize>,
    mods: &mut HashMap<usize, Box<Module>>,
) {
    let this_mod_mut = unsafe { this_mod.as_mut().unwrap().expect_il_mut() };
    for modref in file.modref_tbl.iter() {
        let name = str_heap[modref.name as usize];
        this_mod_mut
            .modrefs
            .push(mods.get_mut(&name).unwrap().as_mut() as *mut Module);
    }
}

pub fn link_typeref(file: &IrFile, this_mod: *mut Module, str_heap: &Vec<usize>) {
    let this_mod_mut = unsafe { this_mod.as_mut().unwrap().expect_il_mut() };
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
}

pub fn link_member_ref(
    file: &IrFile,
    this_mod: *mut Module,
    str_heap: &Vec<usize>,
    str_pool: &Vec<String>,
) {
    let this_mod_mut = unsafe { this_mod.as_mut().unwrap().expect_il_mut() };
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
                        let parent = unsafe { this_mod_mut.typerefs[parent_idx].as_ref().unwrap() };
                        let ps_ty: Vec<BuiltinType> = ps
                            .iter()
                            .map(|p| BuiltinType::from_ir_ele_ty(p, this_mod_mut))
                            .collect();
                        let sig = method_sig(str_pool, name, &ps_ty);
                        if let Some(m) = parent.ee_class.methods.get(&sig) {
                            if unsafe { &m.as_ref().unwrap().ret.ty } == &ret_ty {
                                found = true;
                                this_mod_mut.memberref.push(MemberRef::Method(*m));
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
                        if let Some(f) = unsafe {
                            this_mod_mut.typerefs[parent_idx]
                                .as_ref()
                                .unwrap()
                                .ee_class
                                .fields
                                .get(&name)
                        } {
                            if &sig == unsafe { &f.as_ref().unwrap().ty } {
                                // field found
                                this_mod_mut.memberref.push(MemberRef::Field(*f));
                                found = true;
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
}

pub fn fill_field_info(file: &IrFile, this_mod: *mut Module) {
    let this_mod_mut = unsafe { this_mod.as_mut().unwrap().expect_il_mut() };
    // avoid borrow twice, actually we won't change this_mod_mut.fields so there are no memory issues
    let this_mod_ref = unsafe { this_mod.as_ref().unwrap().expect_il() };

    for (field, field_entry) in this_mod_mut.fields.iter_mut().zip(file.field_tbl.iter()) {
        if let IrSig::Field(f_sig) = &file.blob_heap[field_entry.sig as usize] {
            field.ty = BuiltinType::from_ir_ele_ty(f_sig, this_mod_ref);
        } else {
            panic!();
        }
    }
}

pub fn fill_method_info(file: &IrFile, this_mod: *mut Module) {
    let this_mod_mut = unsafe { this_mod.as_mut().unwrap().expect_il_mut() };
    // avoid borrow twice, actually we won't change this_mod_mut.methods so there are no memory issues
    let this_mod_ref = unsafe { this_mod.as_ref().unwrap().expect_il() };

    for (method, method_entry) in this_mod_mut.methods.iter_mut().zip(file.method_tbl.iter()) {
        let sig = &file.blob_heap[method_entry.sig as usize];
        if let IrSig::Method(_, ps, ret) = sig {
            method.ret.ty = BuiltinType::from_ir_ele_ty(ret, this_mod_ref);
            method.init_ps_ty(ps, this_mod_ref);
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
                        il_impl.init_locals(&locals, this_mod_ref);
                    }
                }
            }
            MethodImpl::Native(_) => {}
        }
    }
}

pub fn link_class_extends(file: &IrFile, this_mod: *mut Module) {
    let this_mod_mut = unsafe { this_mod.as_mut().unwrap().expect_il_mut() };
    // avoid borrow twice, actually we won't change this_mod_mut.types so there are no memory issues
    let this_mod_mut1 = unsafe { this_mod.as_mut().unwrap().expect_il_mut() };
    let this_mod_ref = unsafe { this_mod.as_ref().unwrap().expect_il() };

    for (ty, type_entry) in this_mod_mut.types.iter_mut().zip(file.typedef_tbl.iter()) {
        if let Some((parent_tag, parent_idx)) = type_entry.get_extends() {
            ty.extends = match parent_tag {
                TypeDefOrRef::TypeDef => this_mod_mut1.types[parent_idx].as_mut() as *mut Type,
                TypeDefOrRef::TypeRef => this_mod_ref.typerefs[parent_idx],
                TypeDefOrRef::TypeSpec => unimplemented!(),
            };
        }
    }
}
