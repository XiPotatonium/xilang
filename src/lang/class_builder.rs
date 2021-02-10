use crate::ir::class::class_file::{Classfile, Constant, Field, Instruction, Method};
use crate::ir::flag::Flag;
use crate::ir::ty::VarType;

use std::collections::HashMap;
use std::convert::TryFrom;

pub struct ClassBuilder {
    // use const map to avoid redeclaration
    // str -> utf8 idx
    utf8_map: HashMap<String, u16>,
    // utf8 idx -> String idx
    str_map: HashMap<u16, u16>,
    // utf8 idx -> Class idx
    class_map: HashMap<u16, u16>,
    // (Class idx, NameAndType idx) -> Field idx
    field_map: HashMap<(u16, u16), u16>,
    // (Class idx, NameAndType idx) -> Field idx
    method_map: HashMap<(u16, u16), u16>,
    // (utf8 name idx, utf8 ty idx) -> NameAndType idx
    name_and_type_map: HashMap<(u16, u16), u16>,
    class_file: Classfile,

    codes: Vec<Vec<Instruction>>,
}

impl ClassBuilder {
    pub fn new(class_name: &str, flag: &Flag) -> ClassBuilder {
        let mut ret = ClassBuilder {
            utf8_map: HashMap::new(),
            str_map: HashMap::new(),
            class_map: HashMap::new(),
            field_map: HashMap::new(),
            method_map: HashMap::new(),
            name_and_type_map: HashMap::new(),
            class_file: Classfile::new(flag.flag),
            codes: Vec::new(),
        };

        ret.class_file.this_class = ret.add_const_class(class_name);

        ret
    }

    /// Add a field of this class
    pub fn add_field(&mut self, name: &str, ty: &str, flag: &Flag) -> usize {
        let name_index = self.add_const_utf8(name);
        let descriptor_index = self.add_const_utf8(ty);
        self.class_file.fields.push(Field {
            access_flags: flag.flag,
            name_index,
            descriptor_index,
            attributes: vec![],
        });
        self.class_file.fields.len() - 1
    }

    /// Add a field of this class
    pub fn add_method(&mut self, name: &str, ty: &str, flag: &Flag) -> usize {
        let name_index = self.add_const_utf8(name);
        let descriptor_index = self.add_const_utf8(ty);
        self.class_file.methods.push(Method {
            access_flags: flag.flag,
            name_index,
            descriptor_index,
            attributes: vec![],
        });
        self.class_file.methods.len() - 1
    }
}

// Const values
impl ClassBuilder {
    pub fn add_const_string(&mut self, v: &str) -> u16 {
        let utf8 = self.add_const_utf8(v);
        if let Some(ret) = self.str_map.get(&utf8) {
            *ret
        } else {
            self.class_file.constant_pool.push(Constant::String(utf8));
            let ret = self.class_file.constant_pool.len() as u16;
            self.str_map.insert(utf8, ret);
            ret
        }
    }

    pub fn add_const_utf8(&mut self, v: &str) -> u16 {
        if let Some(ret) = self.utf8_map.get(v) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Utf8(String::from(v)));
            let ret = self.class_file.constant_pool.len() as u16;
            self.utf8_map.insert(String::from(v), ret);
            ret
        }
    }

    pub fn add_const_class(&mut self, class_name: &str) -> u16 {
        let class_name_idx = self.add_const_utf8(class_name);
        if let Some(ret) = self.class_map.get(&class_name_idx) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Class(class_name_idx));
            let ret = self.class_file.constant_pool.len() as u16;
            self.class_map.insert(class_name_idx, ret);
            ret
        }
    }

    fn add_const_name_and_type(&mut self, name: &str, ty: &str) -> u16 {
        let name = self.add_const_utf8(name);
        let ty = self.add_const_utf8(ty);

        if let Some(ret) = self.name_and_type_map.get(&(name, ty)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::NameAndType(name, ty));
            let ret = self.class_file.constant_pool.len() as u16;
            self.name_and_type_map.insert((name, ty), ret);
            ret
        }
    }

    pub fn add_const_fieldref(&mut self, class_name: &str, name: &str, ty: &str) -> u16 {
        let class = self.add_const_class(class_name);
        let name_and_type = self.add_const_name_and_type(name, ty);

        if let Some(ret) = self.field_map.get(&(class, name_and_type)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Fieldref(class, name_and_type));
            let ret = self.class_file.constant_pool.len() as u16;
            self.field_map.insert((class, name_and_type), ret);
            ret
        }
    }

    pub fn add_const_methodref(&mut self, class_name: &str, name: &str, ty: &str) -> u16 {
        let class = self.add_const_class(class_name);
        let name_and_type = self.add_const_name_and_type(name, ty);

        if let Some(ret) = self.method_map.get(&(class, name_and_type)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Methodref(class, name_and_type));
            let ret = self.class_file.constant_pool.len() as u16;
            self.method_map.insert((class, name_and_type), ret);
            ret
        }
    }
}

// instructions
impl ClassBuilder {
    pub fn add_inst_store(&mut self, method_idx: usize, local_ty: &VarType, local_offset: usize) {
        let local_offset = u8::try_from(local_offset).expect("Too large offset");
        self.codes[method_idx].push(match local_ty {
            VarType::Int => Instruction::IStore(local_offset),
            _ => unimplemented!(),
        });
    }
}

// Serde
impl ClassBuilder {
    pub fn serialize(&self, buf: &mut Vec<u8>) {
        unimplemented!();
    }
}
