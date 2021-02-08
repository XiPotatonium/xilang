pub enum VarType {
    Bool,
    U8,
    U16,
    I32,
    F64,
    Void,
    Obj(String),
    Arr(Box<VarType>),
}

impl VarType {
    pub fn slot(&self) -> usize {
        match self {
            Self::Bool | Self::U8 | Self::U16 | Self::I32 | Self::Obj(_) | Self::Arr(_) => 1,
            Self::F64 => 2,
            Self::Void => panic!("Void has no size"),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U16 => 2,
            Self::Bool | Self::I32 | Self::Obj(_) | Self::Arr(_) => 4,
            Self::F64 => 8,
            Self::Void => panic!("Void has no size"),
        }
    }
}
