use xir::attrib::{FieldAttribFlag, TypeAttrib};

use super::super::mem::{addr_addu, to_absolute, MemTag, StaticArea, VTblEntry};
use super::{Field, Method};

use std::mem::size_of;

pub struct Type {
    pub name: usize,
    pub attrib: TypeAttrib,

    pub extends: Option<*mut Type>,

    // ownership of methods and fields is at parent module
    pub methods: Vec<*mut Method>,
    pub fields: Vec<*mut Field>,

    pub vtbl_addr: usize,
    pub instance_field_size: usize,
    pub static_field_size: usize,
}

impl Type {
    pub fn dispose_instance_info(&mut self, static_area: &mut StaticArea) {
        if self.vtbl_addr != 0 {
            // already initialized
            return;
        }

        let mut instance_field_offset = 0;
        let mut static_field_offset = 0;

        if let Some(base) = self.extends {
            let base = unsafe { base.as_mut().unwrap() };
            base.dispose_instance_info(static_area);
        }

        for field in self.fields.iter() {
            let field = unsafe { field.as_mut().unwrap() };

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
        self.instance_field_size = instance_field_offset;
        self.static_field_size = static_field_offset;
        let static_addr = to_absolute(
            MemTag::StaticMem,
            static_area.add_class(
                VTblEntry {
                    class: self as *const Type,
                    num_virt: 0,
                    num_interface: 0,
                },
                vec![],
                vec![],
                static_field_offset,
            ),
        );
        self.vtbl_addr = static_addr;
        // link static field addr
        for field in self.fields.iter() {
            let field = unsafe { field.as_mut().unwrap() };
            if field.attrib.is(FieldAttribFlag::Static) {
                field.addr = addr_addu(static_addr, field.addr);
            }
        }
    }
}
