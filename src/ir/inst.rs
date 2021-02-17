// TODO: Once it becomes stable
// https://github.com/rust-lang/rust/issues/60553
#[derive(Clone, Debug, PartialEq)]
pub enum Inst {
    IConstM1,           // 0X02
    IConst0,            // 0X03
    IConst1,            // 0X04
    IConst2,            // 0X05
    IConst3,            // 0X06
    IConst4,            // 0X07
    IConst5,            // 0X08
    BIPush(i8),         // 0X10
    LdC(u16),           // 0X12
    ILoad(u16),         // 0X15
    ALoad(u16),         // 0X19
    ILoad0,             // 0X1A
    ILoad1,             // 0X1B
    ILoad2,             // 0X1C
    ILoad3,             // 0X1D
    ALoad0,             // 0X2A
    ALoad1,             // 0X2B
    ALoad2,             // 0X2C
    ALoad3,             // 0X2D
    IStore(u16),        // 0X36
    AStore(u16),        // 0X3A
    IStore0,            // 0X3B
    IStore1,            // 0X3C
    IStore2,            // 0X3D
    IStore3,            // 0X3E
    AStore0,            // 0X4B
    AStore1,            // 0X4C
    AStore2,            // 0X4D
    AStore3,            // 0X4E
    Pop,                // 0X57
    Pop2,               // 0X58
    IAdd,               // 0X60
    Return,             // 0XB1
    GetStatic(u16),     // 0XB2
    PutStatic(u16),     // 0XB3
    GetField(u16),      // 0XB4
    PutField(u16),      // 0XB5
    InvokeVirtual(u16), // 0XB6
    InvokeSpecial(u16), // 0XB7
    InvokeStatic(u16),  // 0XB8
    New(u16),           // 0XBB
    ArrayLength,        // 0XBE
}

impl Inst {
    pub fn size(&self) -> u16 {
        match self {
            Self::IConstM1 => 1,
            Self::IConst0 => 1,
            Self::IConst1 => 1,
            Self::IConst2 => 1,
            Self::IConst3 => 1,
            Self::IConst4 => 1,
            Self::IConst5 => 1,
            Self::BIPush(_) => 2,
            Self::LdC(_) => 3,
            Self::ILoad(_) => 3,
            Self::ALoad(_) => 3,
            Self::ILoad0 => 1,
            Self::ILoad1 => 1,
            Self::ILoad2 => 1,
            Self::ILoad3 => 1,
            Self::ALoad0 => 1,
            Self::ALoad1 => 1,
            Self::ALoad2 => 1,
            Self::ALoad3 => 1,
            Self::IStore(_) => 3,
            Self::AStore(_) => 3,
            Self::IStore0 => 1,
            Self::IStore1 => 1,
            Self::IStore2 => 1,
            Self::IStore3 => 1,
            Self::AStore0 => 1,
            Self::AStore1 => 1,
            Self::AStore2 => 1,
            Self::AStore3 => 1,
            Self::Pop => 1,
            Self::Pop2 => 1,
            Self::IAdd => 1,
            Self::Return => 1,
            Self::GetStatic(_) => 3,
            Self::PutStatic(_) => 3,
            Self::GetField(_) => 3,
            Self::PutField(_) => 3,
            Self::InvokeVirtual(_) => 3,
            Self::InvokeSpecial(_) => 3,
            Self::InvokeStatic(_) => 3,
            Self::New(_) => 3,
            Self::ArrayLength => 1,
        }
    }
}
