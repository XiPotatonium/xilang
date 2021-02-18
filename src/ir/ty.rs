use std::fmt;

#[derive(Clone, Eq)]
pub enum RValType {
    Bool,
    U8,
    Char,
    I32,
    F64,
    Void,
    /// class fullname
    Obj(String),
    Array(Box<RValType>),
}

impl RValType {
    pub fn size(&self) -> u16 {
        match self {
            Self::U8 => 1,
            Self::Char => 2,
            Self::I32 | Self::Bool | Self::Obj(_) | Self::Array(_) => 4,
            Self::F64 => 8,
            Self::Void => 0,
        }
    }

    pub fn descriptor(&self) -> String {
        format!("{}", self)
    }
}

impl PartialEq for RValType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool, Self::Bool)
            | (Self::U8, Self::U8)
            | (Self::Char, Self::Char)
            | (Self::I32, Self::I32)
            | (Self::F64, Self::F64)
            | (Self::Void, Self::Void) => true,
            (Self::Obj(class0), Self::Obj(class1)) => class0 == class1,
            _ => false,
        }
    }
}

impl fmt::Display for RValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "Z"),
            Self::U8 => write!(f, "B"),
            Self::Char => write!(f, "C"),
            Self::I32 => write!(f, "I"),
            Self::F64 => write!(f, "D"),
            Self::Void => write!(f, "V"),
            Self::Obj(s) => write!(f, "L{};", s),
            Self::Array(t) => write!(f, "[{}", t),
        }
    }
}

pub fn fn_descriptor(ret_ty: &RValType, ps: &Vec<RValType>) -> String {
    format!(
        "({}){}",
        ps.iter().map(|t| format!("{}", t)).collect::<String>(),
        ret_ty
    )
}
