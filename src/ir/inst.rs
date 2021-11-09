#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    IconstM1,           // 0x02
    Iconst0,            // 0x03
    Iconst1,            // 0x04
    Iconst2,            // 0x05
    Iconst3,            // 0x06
    Iconst4,            // 0x07
    Iconst5,            // 0x08
    Bipush(u8),         // 0x10
    LoadConstant(u8),   // 0x12
    Aload0,             // 0x2A
    Aload1,             // 0x2B
    Aload2,             // 0x2C
    Aload3,             // 0x2D
    Aaload,             // 0x32
    Iadd,               // 0x60
    IfEq(u16),          // 0x99
    IfNe(u16),          // 0x9A
    IfLt(u16),          // 0x9B
    IfGe(u16),          // 0x9C
    IfGt(u16),          // 0x9D
    IfLe(u16),          // 0x9E
    IfIcmpEq(u16),      // 0x9F
    IfIcmpNe(u16),      // 0xA0
    IfIcmpLt(u16),      // 0xA1
    IfIcmpGe(u16),      // 0xA2
    IfIcmpGt(u16),      // 0xA3
    IfIcmpLe(u16),      // 0xA4
    Goto(u16),          // 0xA7
    Return,             // 0xB1
    GetStatic(u16),     // 0xB2
    InvokeVirtual(u16), // 0xB6
    InvokeSpecial(u16), // 0xB7
    InvokeStatic(u16),  // 0xB8
    ArrayLength,        // 0xBE
}

impl Instruction {
    pub fn size(&self) -> usize {
        match *self {
            Instruction::IconstM1 => 1,
            Instruction::Iconst0 => 1,
            Instruction::Iconst1 => 1,
            Instruction::Iconst2 => 1,
            Instruction::Iconst3 => 1,
            Instruction::Iconst4 => 1,
            Instruction::Iconst5 => 1,
            Instruction::Bipush(_) => 2,
            Instruction::LoadConstant(_) => 2,
            Instruction::Aload0 => 1,
            Instruction::Aload1 => 1,
            Instruction::Aload2 => 1,
            Instruction::Aload3 => 1,
            Instruction::Aaload => 1,
            Instruction::Iadd => 1,
            Instruction::IfEq(_) => 3,
            Instruction::IfNe(_) => 3,
            Instruction::IfLt(_) => 3,
            Instruction::IfGe(_) => 3,
            Instruction::IfGt(_) => 3,
            Instruction::IfLe(_) => 3,
            Instruction::IfIcmpEq(_) => 3,
            Instruction::IfIcmpNe(_) => 3,
            Instruction::IfIcmpLt(_) => 3,
            Instruction::IfIcmpGe(_) => 3,
            Instruction::IfIcmpGt(_) => 3,
            Instruction::IfIcmpLe(_) => 3,
            Instruction::Goto(_) => 3,
            Instruction::Return => 1,
            Instruction::GetStatic(_) => 3,
            Instruction::InvokeVirtual(_) => 3,
            Instruction::InvokeSpecial(_) => 3,
            Instruction::InvokeStatic(_) => 3,
            Instruction::ArrayLength => 1,
        }
    }
}
