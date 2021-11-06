use super::inst::Instruction;

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
    Code(
        u16,
        u16,
        u16,
        Vec<Instruction>,
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
