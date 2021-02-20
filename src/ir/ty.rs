use std::fmt;

#[derive(Clone, Eq)]
pub enum IrValType {
    Bool,
    U8,
    Char,
    I32,
    F64,
    Void,
    /// mod fullname, class name
    Obj(String, String),
    Array(Box<IrValType>),
}

impl IrValType {
    pub fn size(&self) -> u16 {
        match self {
            Self::U8 => 1,
            Self::Char => 2,
            Self::I32 | Self::Bool | Self::Obj(_, _) | Self::Array(_) => 4,
            Self::F64 => 8,
            Self::Void => 0,
        }
    }

    pub fn descriptor(&self) -> String {
        format!("{}", self)
    }
}

impl PartialEq for IrValType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool, Self::Bool)
            | (Self::U8, Self::U8)
            | (Self::Char, Self::Char)
            | (Self::I32, Self::I32)
            | (Self::F64, Self::F64)
            | (Self::Void, Self::Void) => true,
            (Self::Obj(mod0, class0), Self::Obj(mod1, class1)) => mod0 == mod1 && class0 == class1,
            _ => false,
        }
    }
}

impl fmt::Display for IrValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "Z"),
            Self::U8 => write!(f, "B"),
            Self::Char => write!(f, "C"),
            Self::I32 => write!(f, "I"),
            Self::F64 => write!(f, "D"),
            Self::Void => write!(f, "V"),
            Self::Obj(m, s) => write!(f, "L{}/{};", m, s),
            Self::Array(t) => write!(f, "[{}", t),
        }
    }
}

pub fn fn_descriptor(ret_ty: &IrValType, ps: &Vec<IrValType>) -> String {
    format!(
        "({}){}",
        ps.iter().map(|t| format!("{}", t)).collect::<String>(),
        ret_ty
    )
}
