use std::fmt;

#[derive(Clone, Eq)]
pub enum RValType {
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Void,
    /// class fullname
    Class(String),
    Array(Box<RValType>),
    /// non-JVM standard
    Tuple(Vec<RValType>),
}

impl RValType {
    pub fn slot(&self) -> u16 {
        match self {
            Self::Boolean
            | Self::Byte
            | Self::Short
            | Self::Char
            | Self::Int
            | Self::Float
            | Self::Class(_)
            | Self::Array(_) => 1,
            Self::Double | Self::Long => 2,
            Self::Tuple(types) => {
                let mut size = 0;
                for ty in types.iter() {
                    size += ty.slot();
                }
                size
            }
            Self::Void => 0,
        }
    }

    pub fn size(&self) -> u16 {
        match self {
            Self::Byte => 1,
            Self::Short | Self::Char => 2,
            Self::Int | Self::Boolean | Self::Float | Self::Class(_) | Self::Array(_) => 4,
            Self::Double | Self::Long => 8,
            Self::Tuple(types) => {
                let mut size = 0u16;
                for ty in types.iter() {
                    size += ty.size();
                }
                size
            }
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
            (Self::Boolean, Self::Boolean)
            | (Self::Byte, Self::Byte)
            | (Self::Char, Self::Char)
            | (Self::Short, Self::Short)
            | (Self::Int, Self::Int)
            | (Self::Long, Self::Long)
            | (Self::Float, Self::Float)
            | (Self::Double, Self::Double)
            | (Self::Void, Self::Void) => true,
            (Self::Class(class0), Self::Class(class1)) => class0 == class1,
            _ => false,
        }
    }
}

impl fmt::Display for RValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Boolean => write!(f, "Z"),
            Self::Byte => write!(f, "B"),
            Self::Char => write!(f, "C"),
            Self::Short => write!(f, "S"),
            Self::Int => write!(f, "I"),
            Self::Long => write!(f, "J"),
            Self::Float => write!(f, "F"),
            Self::Double => write!(f, "D"),
            Self::Void => write!(f, "V"),
            Self::Class(s) => write!(f, "L{};", s),
            Self::Array(t) => write!(f, "[{}", t),
            Self::Tuple(vs) => write!(
                f,
                "({}",
                vs.iter().map(|t| format!("{}", t)).collect::<String>()
            ),
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
