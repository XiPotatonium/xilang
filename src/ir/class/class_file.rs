use super::super::inst::Inst;

use std::ops::Index;

const CAFEBABE: u32 = 0xCAFEBABE;
const MAJOR_VERSION: u16 = 52;
const MINOR_VERSION: u16 = 0;

#[derive(Clone, Debug, PartialEq)]
pub struct ClassFile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<Constant>,
    pub access_flags: u16,
    pub this_class: u16,
    pub interfaces: Vec<IrInterface>,
    pub fields: Vec<IrField>,
    pub methods: Vec<IrMethod>,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Utf8(String),          // 1
    Integer(i32),          // 3
    Class(u16),            // 7
    String(u16),           // 8
    Fieldref(u16, u16),    // 9
    Methodref(u16, u16),   // 10
    NameAndType(u16, u16), // 12
}

#[derive(Clone, Debug, PartialEq)]
pub struct IrInterface;

#[derive(Clone, Debug, PartialEq)]
pub struct IrField {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IrMethod {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
    Code(
        // index of "Code"
        u16,
        // max stacks
        u16,
        Vec<Inst>,
        Vec<ExceptionTableEntry>,
        Vec<Attribute>,
    ),
    LineNumberTable(u16, Vec<LineNumberTableEntry>),
    SourceFile(u16, u16),
    StackMapTable(u16, Vec<StackMapFrame>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionTableEntry;

#[derive(Clone, Debug, PartialEq)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StackMapFrame {
    SameFrame(u8),
    SameLocals1StackItemFrame(u8, VerificationType),
    SameLocals1StackItemFrameExtended(u16, VerificationType),
    ChopFrame(u8, u16),
    SameFrameExtended(u16),
    AppendFrame(u8, u16, Vec<VerificationType>),
    FullFrame(u16, Vec<VerificationType>, Vec<VerificationType>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerificationType {
    Top,                // 0
    Integer,            // 1
    Float,              // 2
    Long,               // 3
    Double,             // 4
    Null,               // 5
    UninitializedThis,  // 6
    Object(u16),        // 7
    Uninitialized(u16), // 8
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
            attributes: vec![],
        }
    }
}

impl Index<u16> for ClassFile {
    type Output = Constant;

    fn index(&self, idx: u16) -> &Self::Output {
        &self.constant_pool[idx as usize]
    }
}
