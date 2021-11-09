use std::collections::HashMap;

use ir::flags::{ClassFlags, FieldFlags, MethodFlags};
use ir::Field as IrField;
use ir::Method as IrMethod;
use ir::{ClassFile, Constant};

pub struct ClassFileHelper {
    pub file: ClassFile,

    /// (name_index, descriptor_index) -> index in field table
    field_map: HashMap<(u16, u16), usize>,
    /// (name_index, descriptor_index) -> index in method table
    method_map: HashMap<(u16, u16), usize>,
    /// name utf8 constant -> a class constant
    classref_map: HashMap<u16, u16>,
    /// str -> a utf-8 constant
    utf8_map: HashMap<String, u16>,
}

impl ClassFileHelper {
    pub fn new(name: &str, flags: ClassFlags) -> ClassFileHelper {
        let mut helper = ClassFileHelper {
            file: ClassFile::new(),
            field_map: HashMap::new(),
            method_map: HashMap::new(),
            classref_map: HashMap::new(),
            utf8_map: HashMap::new(),
        };
        // Fill this class
        helper.file.access_flags = flags.0;
        helper.file.this_class = helper.add_classref(name);
        // Fill super class
        // WARNING: Hard coding
        helper.file.super_class = helper.add_classref("java/lang/Object");

        helper
    }

    pub fn add_utf8_str(&mut self, utf8: &str) -> u16 {
        if let Some(idx) = self.utf8_map.get(utf8) {
            *idx
        } else {
            self.file
                .constant_pool
                .push(Constant::Utf8(utf8.to_owned()));
            let idx = self.file.constant_pool.len() as u16;
            self.utf8_map.insert(utf8.to_owned(), idx);
            idx
        }
    }

    pub fn add_utf8_string(&mut self, utf8: String) -> u16 {
        if let Some(idx) = self.utf8_map.get(&utf8) {
            *idx
        } else {
            self.file
                .constant_pool
                .push(Constant::Utf8(utf8.to_owned()));
            let idx = self.file.constant_pool.len() as u16;
            self.utf8_map.insert(utf8, idx);
            idx
        }
    }

    pub fn add_classref(&mut self, name: &str) -> u16 {
        let name_idx = self.add_utf8_str(name);
        if let Some(idx) = self.classref_map.get(&name_idx) {
            *idx
        } else {
            self.file.constant_pool.push(Constant::Class(name_idx));
            let idx = self.file.constant_pool.len() as u16;
            self.classref_map.insert(name_idx, idx);
            idx
        }
    }

    pub fn add_methodref(&mut self, name: &str, descriptor: &str) -> u16 {
        unimplemented!()
    }

    pub fn add_fieldref(&mut self, name: &str, descriptor: &str) -> u16 {
        unimplemented!()
    }

    pub fn add_method(&mut self, name: &str, descriptor: String, flags: MethodFlags) -> usize {
        let name_index = self.add_utf8_str(name);
        let descriptor_index = self.add_utf8_string(descriptor);
        if let Some(idx) = self.method_map.get(&(name_index, descriptor_index)) {
            *idx
        } else {
            let idx = self.file.methods.len();
            self.method_map.insert((name_index, descriptor_index), idx);
            self.file.methods.push(IrMethod {
                access_flags: flags.0,
                name_index,
                descriptor_index,
                attributes: vec![],
            });
            idx
        }
    }

    pub fn add_field(&mut self, name: &str, descriptor: String, flags: FieldFlags) -> usize {
        let name_index = self.add_utf8_str(name);
        let descriptor_index = self.add_utf8_string(descriptor);
        if let Some(idx) = self.field_map.get(&(name_index, descriptor_index)) {
            *idx
        } else {
            let idx = self.file.fields.len();
            self.field_map.insert((name_index, descriptor_index), idx);
            self.file.fields.push(IrField {
                access_flags: flags.0,
                name_index,
                descriptor_index,
                attributes: vec![],
            });
            idx
        }
    }
}
