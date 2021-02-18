use super::inst::Inst;

use std::ops::Index;

const CAFEBABE: u32 = 0xCAFEBABE;
const MAJOR_VERSION: u16 = 1;
const MINOR_VERSION: u16 = 0;

#[derive(Clone, Debug, PartialEq)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<Constant>,
    pub access_flags: u16,
    pub this_class: u32,
    pub interfaces: Vec<IrInterface>,
    pub fields: Vec<IrField>,
    pub methods: Vec<IrMethod>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Utf8(String),          // 1
    Class(u32),            // 7
    String(u32),           // 8
    Fieldref(u32, u32),    // 9
    Methodref(u32, u32),   // 10
    NameAndType(u32, u32), // 12
}

#[derive(Clone, Debug, PartialEq)]
pub struct IrInterface;

#[derive(Clone, Debug, PartialEq)]
pub struct IrField {
    pub access_flags: u16,
    pub name_index: u32,
    pub descriptor_index: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IrMethod {
    pub access_flags: u16,
    pub name_index: u32,
    pub descriptor_index: u32,

    pub locals_stack: u16,
    pub insts: Vec<Inst>,
    pub exception: Vec<ExceptionTableEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionTableEntry;

#[derive(Clone, Debug, PartialEq)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

impl ClassFile {
    pub fn new(access_flags: u16) -> ClassFile {
        ClassFile {
            magic: CAFEBABE,
            minor_version: MINOR_VERSION,
            major_version: MAJOR_VERSION,
            constant_pool: vec![],
            access_flags: access_flags,
            this_class: 0,
            interfaces: vec![],
            fields: vec![],
            methods: vec![],
        }
    }
}

impl Index<u32> for ClassFile {
    type Output = Constant;

    fn index(&self, idx: u32) -> &Self::Output {
        &self.constant_pool[idx as usize - 1]
    }
}
