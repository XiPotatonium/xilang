pub mod arr;

use xir::attrib::TypeAttrib;

use super::super::util::ptr::NonNull;
use super::{Field, ILModule, MethodDesc};

use std::collections::HashMap;
use std::ptr;
use std::rc::Rc;

pub enum TypeInitState {
    Uninitialized,
    InitializingMemLayout,
    InitializingVtbl,
    Initialized,
}

/// information used in loading
pub struct EEClass {
    pub is_value: bool,
    pub init_state: TypeInitState,
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
                init_state: TypeInitState::Uninitialized,
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

    pub fn fullname(&self, str_pool: &Vec<String>) -> String {
        format!(
            "{}/{}",
            unsafe { self.module.as_ref().fullname(str_pool) },
            &str_pool[self.name]
        )
    }
}
