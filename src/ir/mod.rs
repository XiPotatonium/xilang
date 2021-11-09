#[macro_use]

pub mod flags;
mod attrib;
pub mod constant;
pub mod inst;

use attrib::Attribute;
use constant::Constant;

const CAFEBABE: u32 = 0xCAFEBABE;
const MAJOR_VERSION: u16 = 61;
const MINOR_VERSION: u16 = 0;

#[derive(Clone, Debug)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<Constant>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

impl ClassFile {
    pub fn new() -> ClassFile {
        ClassFile {
            magic: CAFEBABE,
            minor_version: MINOR_VERSION,
            major_version: MAJOR_VERSION,
            constant_pool: vec![],
            access_flags: 0,
            this_class: 0,
            super_class: 0,
            interfaces: vec![],
            fields: vec![],
            methods: vec![],
            attributes: vec![],
        }
    }

    pub fn get_constant(&self, index: u16) -> &Constant {
        &self.constant_pool[index as usize - 1]
    }

    pub fn get_string(&self, index: u16) -> &str {
        let val = self.get_constant(index);
        match *val {
            Constant::Utf8(ref str) => str,
            _ => panic!("Wanted string, found {:?}", val),
        }
    }
}
