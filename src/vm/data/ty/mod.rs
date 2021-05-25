use xir::attrib::{FieldAttribFlag, MethodAttribFlag, TypeAttrib};

use super::super::util::ptr::NonNull;
use super::{Field, ILModule, MethodDesc};

use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;

/// information used in loading
pub struct EEClass {
    pub initialized: bool,
    pub is_value: bool,
    /// key method.sig()
    pub methods: HashMap<String, NonNull<MethodDesc>>,
    /// key: name
    pub fields: HashMap<usize, NonNull<Field>>,
}

pub struct Type {
    pub name: usize,
    pub attrib: TypeAttrib,

    pub ee_class: Rc<EEClass>,

    pub extends: *mut Type,
    pub module: NonNull<ILModule>,

    /// instance field size
    pub basic_instance_size: usize,
    pub static_fields: Vec<u8>,

    pub vtbl: Vec<NonNull<MethodDesc>>,
}

impl Type {
    pub fn new(
        module: NonNull<ILModule>,
        name: usize,
        attrib: TypeAttrib,
        fields: HashMap<usize, NonNull<Field>>,
        methods: HashMap<String, NonNull<MethodDesc>>,
    ) -> Type {
        Type {
            name,
            attrib,
            ee_class: Rc::new(EEClass {
                initialized: false,
                is_value: false,
                fields,
                methods,
            }),
            module,
            // fill in link stage
            extends: ptr::null_mut(),
            // fill in allocation stage
            basic_instance_size: 0,
            static_fields: vec![],

            vtbl: vec![],
        }
    }

    pub fn dispose_instance_info(&mut self) {
        if self.ee_class.initialized {
            // already initialized
            return;
        }

        // check if type is a value type or enum
        let mut base_ptr = self.extends;
        while let Some(base) = unsafe { base_ptr.as_ref() } {
            base_ptr = base.extends;
        }

        let mut instance_field_offset = 0;
        let mut static_field_offset = 0;

        if let Some(base) = unsafe { self.extends.as_mut() } {
            base.dispose_instance_info();
            // base fields
            instance_field_offset += base.basic_instance_size;
            // base methods
            for method_slot in base.vtbl.iter() {
                self.vtbl.push(*method_slot);
            }
        }

        for (field_name, field) in self.ee_class.fields.iter() {
            let field = unsafe { field.as_mut() };

            // determine field relative offset
            // no alignment
            let field_heap_size = field.ty.byte_size();
            if field.attrib.is(FieldAttribFlag::Static) {
                field.offset = static_field_offset;
                static_field_offset += field_heap_size;
            } else {
                field.offset = instance_field_offset;

                let mut base_ptr = self.extends;
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
                    instance_field_offset += field_heap_size;
                }
            }
        }

        // allocate static field space
        self.basic_instance_size = instance_field_offset;
        self.static_fields.resize(static_field_offset, 0);
        // link static field addr
        for field in self.ee_class.fields.values() {
            let field = unsafe { field.as_mut() };
            if field.attrib.is(FieldAttribFlag::Static) {
                field.addr =
                    (self.static_fields.as_mut_ptr() as *mut u8).wrapping_add(field.offset);
            }
        }

        for (method_sig, method) in self.ee_class.methods.iter() {
            let method = unsafe { method.as_mut() };
            method.slot = self.vtbl.len();

            // TODO: for NewSlot methods always allocate a new slot

            if !method.attrib.is(MethodAttribFlag::NewSlot) {
                let mut base_ptr = self.extends;
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

            let method_ptr = NonNull::new(method as *mut MethodDesc).unwrap();
            if method.slot == self.vtbl.len() {
                // alloc new slot
                self.vtbl.push(method_ptr);
            } else {
                // use matched slot
                self.vtbl[method.slot] = method_ptr;
            }
        }

        Rc::get_mut(&mut self.ee_class).unwrap().initialized = true;
    }

    pub fn fullname(&self, str_pool: &Vec<String>) -> String {
        format!(
            "{}/{}",
            unsafe { self.module.as_ref().fullname(str_pool) },
            &str_pool[self.name]
        )
    }
}
