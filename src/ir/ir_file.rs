use super::inst::Inst;

use std::ops::Index;

const MAJOR_VERSION: u16 = 1;
const MINOR_VERSION: u16 = 0;

#[derive(Debug)]
pub struct ModuleFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub module_name: u32,
    pub constant_pool: Vec<Constant>,
    pub imports: Vec<u32>,
    pub sub_mods: Vec<u32>,
    pub classes: Vec<IrClass>,
    pub fields: Vec<IrField>,
    pub methods: Vec<IrMethod>,
}

#[derive(Debug)]
pub enum Constant {
    Utf8(String),          // 1
    Class(u32),            // 7
    String(u32),           // 8
    Fieldref(u32, u32),    // 9
    Methodref(u32, u32),   // 10
    NameAndType(u32, u32), // 12
}

#[derive(Debug)]
pub struct IrClass {
    pub flag: u32,
    pub name_idx: u32,
}

#[derive(Debug)]
pub struct IrField {
    pub class_idx: u16,

    pub flag: u16,
    pub name_idx: u32,
    pub descriptor_idx: u32,
}

#[derive(Debug)]
pub struct IrMethod {
    pub class_idx: u16,

    pub flag: u16,
    pub name_idx: u32,
    pub descriptor_idx: u32,

    pub locals: u16,
    pub insts: Vec<Inst>,
    pub exception: Vec<ExceptionTableEntry>,
}

#[derive(Debug)]
pub struct ExceptionTableEntry;

#[derive(Debug)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

impl ModuleFile {
    pub fn new() -> ModuleFile {
        ModuleFile {
            minor_version: MINOR_VERSION,
            major_version: MAJOR_VERSION,
            module_name: 0,
            constant_pool: vec![],
            sub_mods: vec![],
            imports: vec![],
            classes: vec![],
            fields: vec![],
            methods: vec![],
        }
    }
}

impl Index<u32> for ModuleFile {
    type Output = Constant;

    fn index(&self, idx: u32) -> &Self::Output {
        &self.constant_pool[idx as usize - 1]
    }
}
