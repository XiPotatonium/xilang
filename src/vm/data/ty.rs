use xir::attrib::{FieldAttribFlag, TypeAttrib};

use super::{Field, Method};

use std::ptr;

pub struct Type {
    pub initialized: bool,

    pub name: usize,
    pub attrib: TypeAttrib,

    pub extends: *mut Type,

    // ownership of methods and fields is at parent module
    pub methods: Vec<*mut Method>,

    pub instance_field_size: usize,

    pub fields: Vec<*mut Field>,
    pub static_fields: Vec<u8>,

    pub vtbl: Vec<*const Method>,
}

impl Type {
    pub fn new(
        name: usize,
        attrib: TypeAttrib,
        fields: Vec<*mut Field>,
        methods: Vec<*mut Method>,
    ) -> Type {
        Type {
            initialized: false,
            name,
            attrib,
            fields,
            methods,
            // fill in link stage
            extends: ptr::null_mut(),
            // fill in allocation stage
            instance_field_size: 0,
            static_fields: vec![],

            vtbl: vec![],
        }
    }

    pub fn dispose_instance_info(&mut self) {
        if self.initialized {
            // already initialized
            return;
        }

        let mut instance_field_offset = 0;
        let mut static_field_offset = 0;

        if let Some(base) = unsafe { self.extends.as_mut() } {
            base.dispose_instance_info();
        }

        for field in self.fields.iter() {
            let field = unsafe { field.as_mut().unwrap() };

            // determine field relative offset
            // no alignment
            let field_heap_size = field.ty.byte_size();
            if field.attrib.is(FieldAttribFlag::Static) {
                field.offset = static_field_offset;
                static_field_offset += field_heap_size;
            } else {
                field.offset = instance_field_offset;
                instance_field_offset += field_heap_size;
            }
        }

        // allocate static field space
        self.instance_field_size = instance_field_offset;
        self.static_fields.resize(static_field_offset, 0);
        // link static field addr
        for field in self.fields.iter() {
            let field = unsafe { field.as_mut().unwrap() };
            if field.attrib.is(FieldAttribFlag::Static) {
                field.addr =
                    (self.static_fields.as_mut_ptr() as *mut u8).wrapping_add(field.offset);
            }
        }

        self.initialized = true;
    }
}
