#[derive(Clone)]
pub enum VarType {
    Bool,
    U8,
    U16,
    I32,
    F64,
    Void,
    Obj(String),
    Arr(Box<VarType>),
    Tuple(Vec<VarType>),
}

impl VarType {
    pub fn slot(&self) -> usize {
        match self {
            Self::Bool | Self::U8 | Self::U16 | Self::I32 | Self::Obj(_) | Self::Arr(_) => 1,
            Self::F64 => 2,
            Self::Tuple(types) => {
                let mut size = 0usize;
                for ty in types.iter() {
                    size += ty.slot();
                }
                size
            }
            Self::Void => 0,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U16 => 2,
            Self::Bool | Self::I32 | Self::Obj(_) | Self::Arr(_) => 4,
            Self::F64 => 8,
            Self::Tuple(types) => {
                let mut size = 0usize;
                for ty in types.iter() {
                    size += ty.size();
                }
                size
            }
            Self::Void => 0,
        }
    }
}
