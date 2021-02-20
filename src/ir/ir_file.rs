use super::inst::Inst;

use std::ops::Index;

pub const MAJOR_VERSION: u16 = 1;
pub const MINOR_VERSION: u16 = 0;

#[derive(Debug)]
pub struct IrFile {
    pub minor_version: u16,
    pub major_version: u16,

    pub mod_name: u32,
    pub entrypoint: u32,

    pub constant_pool: Vec<Constant>,

    pub class_tbl: Vec<IrClass>,
    pub field_tbl: Vec<IrField>,
    pub method_tbl: Vec<IrMethod>,
    pub codes: Vec<Vec<Inst>>,
}

#[derive(Debug)]
pub enum Constant {
    /// 0x01
    Utf8(String),
    /// 0x07
    Class(u32, u32),
    /// 0x08 <utf8>
    String(u32),
    /// 0x09 <class | mod> <name_and_type>
    Fieldref(u32, u32),
    /// 0x0A <class | mod> <name_and_type>
    Methodref(u32, u32),
    /// 0x0C <name> <descriptor>
    NameAndType(u32, u32),
    /// 0x13 <name>
    Mod(u32),
}

#[derive(Debug)]
pub struct IrMod {
    pub name: u32,

    /// index of codes
    pub entrypoint: u32,
}

#[derive(Debug)]
pub struct IrClass {
    pub name: u32,
    pub flag: u32,

    pub fields: u32,
    pub methods: u32,
}

#[derive(Debug)]
pub struct IrField {
    pub name: u32,
    pub descriptor: u32,

    pub flag: u16,
}

#[derive(Debug)]
pub struct IrMethod {
    pub name: u32,
    pub descriptor: u32,

    pub flag: u16,
    pub locals: u16,
}

impl IrFile {
    pub fn new() -> IrFile {
        IrFile {
            minor_version: MINOR_VERSION,
            major_version: MAJOR_VERSION,

            mod_name: 0,
            entrypoint: 0,

            constant_pool: vec![],

            class_tbl: vec![],
            field_tbl: vec![],
            method_tbl: vec![],

            codes: vec![],
        }
    }

    pub fn mod_name(&self) -> Option<&str> {
        if self.mod_name == 0 {
            None
        } else {
            match &self[self.mod_name] {
                Constant::Utf8(s) => Some(s),
                _ => panic!("Invalid file format"),
            }
        }
    }
}

impl Index<u32> for IrFile {
    type Output = Constant;

    fn index(&self, idx: u32) -> &Self::Output {
        &self.constant_pool[idx as usize - 1]
    }
}
