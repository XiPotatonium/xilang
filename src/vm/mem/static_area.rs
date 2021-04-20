use super::super::data::{Method, Type};

use std::mem::size_of;

pub struct StaticArea {
    next_obj_offset: usize,
    data: Vec<u8>,
}

impl StaticArea {
    pub fn new(size: usize) -> StaticArea {
        StaticArea {
            data: vec![0; size],
            next_obj_offset: 0,
        }
    }

    /// Add a class
    ///
    /// virt and interface are not implemented

    /// return: offset
    pub unsafe fn add_class(
        &mut self,
        vtbl_entry: VTblEntry,
        virts: Vec<*const Method>,
        interfaces: Vec<*const Method>,
        static_size: usize,
    ) -> usize {
        assert_eq!(vtbl_entry.num_virt, virts.len());
        assert_eq!(vtbl_entry.num_interface, interfaces.len());
        let ret = self.next_obj_offset;
        let total_size = size_of::<VTblEntry>()
            + (virts.len() + interfaces.len()) * size_of::<usize>()
            + static_size;
        if total_size + ret >= self.data.len() {
            panic!("No enough space for static data");
        }
        self.next_obj_offset += total_size;
        let ptr = &mut self.data[ret] as *mut u8 as *mut VTblEntry;
        *ptr.as_mut().unwrap() = vtbl_entry;
        for _ in virts.iter() {
            unimplemented!();
        }
        for _ in interfaces.iter() {
            unimplemented!();
        }
        // no gc, so content of mem are all zeros
        ret
    }
}

#[derive(Clone, Copy)]
pub struct VTblEntry {
    pub class: *const Type,
    pub num_virt: usize,
    pub num_interface: usize,
}
