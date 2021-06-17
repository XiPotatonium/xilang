use xir::attrib::{FieldAttribFlag, MethodAttribFlag};
use xir::member::MemberRefParent;
use xir::sig::IrSig;
use xir::ty::{ResolutionScope, TypeDefOrRef};

use super::super::data::{
    method_str_desc, BuiltinType, ILModule, Local, MemberRef, MethodImpl, Module, Type,
    TypeInitState, REF_SIZE,
};
use super::super::util::ptr::NonNull;

use std::collections::HashMap;
use std::rc::Rc;

pub fn link_modref(module: NonNull<Module>, mods: &HashMap<usize, Box<Module>>) {
    match unsafe { module.as_mut() } {
        Module::IL(il_mod_mut) => {
            for modref in il_mod_mut.ir_file.modref_tbl.iter() {
                let name = il_mod_mut.str_heap[modref.name as usize];
                il_mod_mut.modrefs.push(
                    NonNull::new(mods.get(&name).unwrap().as_ref() as *const Module as *mut Module)
                        .unwrap(),
                );
            }
        }
        Module::Native(_) => {}
    }
}

pub fn link_typeref(this_mod: NonNull<Module>) {
    let this_mod_mut = unsafe { this_mod.as_mut().expect_il_mut() };
    for typeref in this_mod_mut.ir_file.typeref_tbl.iter() {
        let name = this_mod_mut.str_heap[typeref.name as usize];
        let (parent_tag, parent_idx) = typeref.get_parent();
        match parent_tag {
            ResolutionScope::Mod => unimplemented!(), // this is ok
            ResolutionScope::ModRef => {
                let parent = unsafe { this_mod_mut.modrefs[parent_idx].as_mut() };
                let ty = parent
                    .expect_il_mut()
                    .types
                    .iter_mut()
                    .find(|c| c.as_ref().name == name);
                if let Some(ty) = ty {
                    this_mod_mut
                        .typerefs
                        .push(NonNull::new(ty.as_mut() as *mut Type).unwrap());
                } else {
                    panic!("External symbol not found");
                }
            }
            ResolutionScope::TypeRef => unimplemented!(),
        }
    }
}

/// fill ty.module and ty.extends
pub fn link_type_info(this_mod: NonNull<Module>) {
    let this_mod_mut = unsafe { this_mod.as_mut().expect_il_mut() };
    let il_mod_ptr = NonNull::new(this_mod_mut as *mut ILModule).unwrap();
    // avoid borrow twice, actually we won't change this_mod_mut.types so there are no memory issues
    let this_mod_mut1 = unsafe { this_mod.as_mut().expect_il_mut() };
    let this_mod_ref = unsafe { this_mod.as_ref().expect_il() };

    for (ty, type_entry) in this_mod_mut
        .types
        .iter_mut()
        .zip(this_mod_ref.ir_file.typedef_tbl.iter())
    {
        ty.module = il_mod_ptr;
        if let Some((parent_tag, parent_idx)) = type_entry.get_extends() {
            ty.extends = match parent_tag {
                TypeDefOrRef::TypeDef => this_mod_mut1.types[parent_idx].as_mut() as *mut Type,
                TypeDefOrRef::TypeRef => this_mod_ref.typerefs[parent_idx].as_ptr(),
                TypeDefOrRef::TypeSpec => unimplemented!(),
            };
        }
    }
}

/// init ee_class, fill field info, calc memory layout and alloc static space
pub fn calc_type_mem_layout(ty: &mut Type, str_pool: &Vec<String>) {
    match ty.ee_class.init_state {
        TypeInitState::Uninitialized => {}
        TypeInitState::InitializingMemLayout => panic!(
            "Cyclic loop detected in type dependency graph (in calc mem layout of {})",
            ty.fullname(str_pool)
        ),
        TypeInitState::InitializingVtbl => return,
        TypeInitState::Initialized => unreachable!(),
    }

    Rc::get_mut(&mut ty.ee_class).unwrap().init_state = TypeInitState::InitializingMemLayout;

    // check if type is a value type or enum
    {
        let mut base_ptr = ty.extends;
        while let Some(base) = unsafe { base_ptr.as_ref() } {
            if str_pool[unsafe { base.module.as_ref() }.fullname] == "std"
                && str_pool[base.name] == "ValueType"
            {
                Rc::get_mut(&mut ty.ee_class).unwrap().is_value = true;
            }
            base_ptr = base.extends;
        }
    }

    let mut instance_field_offset = 0;
    if let Some(base) = unsafe { ty.extends.as_mut() } {
        calc_type_mem_layout(base, str_pool);
        // base fields
        instance_field_offset += base.basic_instance_size;
    }

    let module = unsafe { ty.module.as_ref() };
    for (field_name, field) in ty.ee_class.fields.iter() {
        let field = unsafe { field.as_mut() };

        // fill field.ty
        if let IrSig::Field(field_ty) =
            &module.ir_file.blob_heap[module.ir_file.field_tbl[field.index].sig as usize]
        {
            field.ty = BuiltinType::from_type_sig(field_ty, module);
        } else {
            unreachable!();
        }

        // determine field relative offset
        // no alignment
        if field.attrib.is(FieldAttribFlag::Static) {
            continue;
        } else {
            match field.ty {
                BuiltinType::Value(t) => {
                    // ty.byte_size must be called after Type's memory layout is determined when ty is Value
                    calc_type_mem_layout(unsafe { t.as_mut() }, str_pool);
                }
                BuiltinType::GenericInst(_, _, _) => todo!(),
                _ => {}
            }
            field.offset = instance_field_offset;

            let mut base_ptr = ty.extends;
            while let Some(base) = unsafe { base_ptr.as_ref() } {
                if let Some(candidate) = base.ee_class.fields.get(field_name) {
                    let candidate = unsafe { candidate.as_ref() };
                    if candidate.ty == field.ty {
                        // sig hit
                        field.offset = candidate.offset;
                        break;
                    }
                }
                base_ptr = base.extends;
            }

            if field.offset == instance_field_offset {
                // alloc new slot
                instance_field_offset += field.ty.byte_size();
            }
        }
    }

    ty.basic_instance_size = instance_field_offset;
    // Initializing is done after layout is determined
    Rc::get_mut(&mut ty.ee_class).unwrap().init_state = TypeInitState::InitializingVtbl;

    // allocate static field space
    let mut static_field_offset = 0;
    for (_, field) in ty.ee_class.fields.iter() {
        let field = unsafe { field.as_mut() };

        // determine field relative offset
        // no alignment
        if field.attrib.is(FieldAttribFlag::Static) {
            match field.ty {
                BuiltinType::Value(t) => {
                    // ty.byte_size must be called after Type's memory layout is determined when ty is Value
                    calc_type_mem_layout(unsafe { t.as_mut() }, str_pool);
                }
                BuiltinType::GenericInst(_, _, _) => todo!(),
                _ => {}
            }
            field.offset = static_field_offset;
            static_field_offset += field.ty.byte_size();
        } else {
            continue;
        }
    }

    ty.static_fields.resize(static_field_offset, 0);
    // link static field addr
    for field in ty.ee_class.fields.values() {
        let field = unsafe { field.as_mut() };
        if field.attrib.is(FieldAttribFlag::Static) {
            field.addr = (ty.static_fields.as_mut_ptr() as *mut u8).wrapping_add(field.offset);
        }
    }
}

/// alloc vtbl, calc param size and local size, must be called after all types' memory layouts have been determined
pub fn fill_type_method_info(ty: &mut Type) {
    match ty.ee_class.init_state {
        TypeInitState::Uninitialized | TypeInitState::InitializingMemLayout => unreachable!(),
        TypeInitState::InitializingVtbl => {}
        TypeInitState::Initialized => return,
    }

    if let Some(base) = unsafe { ty.extends.as_mut() } {
        fill_type_method_info(base);
        // base methods
        for method_slot in base.vtbl.iter() {
            ty.vtbl.push(*method_slot);
        }
    }

    for (method_sig, method_ptr) in ty.ee_class.methods.iter() {
        let method = unsafe { method_ptr.as_mut() };
        let module = unsafe { method.ctx.as_ref() }.expect_il();

        // handle param info and return info
        if let IrSig::Method(_, ps_ty, ret_ty) =
            &module.ir_file.blob_heap[module.ir_file.method_tbl[method.index].sig as usize]
        {
            // calc method param size
            assert_eq!(method.ps.len(), ps_ty.len());
            let mut offset = if method.is_static() { 0 } else { REF_SIZE };
            for (p, p_ty) in method.ps.iter_mut().zip(ps_ty.iter()) {
                p.ty = BuiltinType::from_param(p_ty, module);
                p.offset = offset;
                // no alignment
                offset += p.ty.byte_size();
            }
            method.ps_size = offset;
            method.ret.ty = BuiltinType::from_ret(ret_ty, module);
        } else {
            unreachable!();
        }

        // handle method impl info
        match &mut method.method_impl {
            MethodImpl::IL(method_impl) => {
                let body = &module.ir_file.codes[method_impl.index];
                if body.locals != 0 {
                    if let IrSig::LocalVar(local_types) = &module.ir_file.blob_heap
                        [module.ir_file.stand_alone_sig_tbl[body.locals as usize - 1].sig as usize]
                    {
                        let mut local_size: usize = 0;
                        for local_ty in local_types.iter() {
                            let local_ty = BuiltinType::from_local(local_ty, module);
                            let size = local_ty.byte_size();
                            method_impl.locals.push(Local {
                                ty: local_ty,
                                offset: local_size,
                            });
                            local_size += size;
                        }
                        method_impl.locals_size = local_size;
                    } else {
                        unreachable!();
                    }
                }
            }
            MethodImpl::Native(_) => {}
            MethodImpl::Runtime(_) => {
                // maybe we can generate internal calls here
            }
        }

        // alloc slot
        method.slot = ty.vtbl.len();
        // try override if method is virtual and not marked NewSlot
        if !method.attrib.is(MethodAttribFlag::NewSlot)
            && method.attrib.is(MethodAttribFlag::Virtual)
        {
            let mut base_ptr = ty.extends;
            while let Some(base) = unsafe { base_ptr.as_ref() } {
                if let Some(candidate) = base.ee_class.methods.get(method_sig) {
                    let candidate = unsafe { candidate.as_ref() };
                    if candidate.ret.ty == method.ret.ty {
                        // sig hit
                        method.slot = candidate.slot;
                        break;
                    }
                }
                base_ptr = base.extends;
            }
        }

        if method.slot == ty.vtbl.len() {
            // alloc new slot
            ty.vtbl.push(method_ptr.clone());
        } else if method.slot > ty.vtbl.len() {
            panic!("Error alloc slot for {}", method_sig);
        } else {
            // use matched slot
            ty.vtbl[method.slot] = method_ptr.clone();
        }
    }

    Rc::get_mut(&mut ty.ee_class).unwrap().init_state = TypeInitState::Initialized;
}

pub fn link_memberref(module: &mut ILModule, str_pool: &Vec<String>) {
    for memberref in module.ir_file.memberref_tbl.iter() {
        let name = module.str_heap[memberref.name as usize];
        let mut found = false;

        let sig = &module.ir_file.blob_heap[memberref.sig as usize];
        let (parent_tag, parent_idx) = memberref.get_parent();
        let parent_idx = parent_idx as usize - 1;

        match sig {
            IrSig::Method(_, ps, ret) => {
                // this member ref is a function
                let ret_ty = BuiltinType::from_ret(ret, module);

                match parent_tag {
                    MemberRefParent::TypeRef => {
                        let parent = unsafe { module.typerefs[parent_idx].as_ref() };
                        let ps_ty: Vec<BuiltinType> = ps
                            .iter()
                            .map(|p| BuiltinType::from_param(p, module))
                            .collect();
                        let sig = method_str_desc(str_pool, name, &ps_ty);
                        if let Some(m) = parent.ee_class.methods.get(&sig) {
                            if unsafe { &m.as_ref().ret.ty } == &ret_ty {
                                found = true;
                                module.memberref.push(MemberRef::Method(*m));
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
                let sig = BuiltinType::from_type_sig(f_sig, module);
                match parent_tag {
                    MemberRefParent::TypeRef => {
                        // check if parent has this field
                        if let Some(f) = unsafe {
                            module.typerefs[parent_idx]
                                .as_ref()
                                .ee_class
                                .fields
                                .get(&name)
                        } {
                            if &sig == unsafe { &f.as_ref().ty } {
                                // field found
                                module.memberref.push(MemberRef::Field(*f));
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
            _ => unreachable!(),
        }

        if !found {
            panic!("External symbol not found");
        }
    }
}
