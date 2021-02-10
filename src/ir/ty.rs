use std::fmt;

#[derive(Clone)]
pub enum VarType {
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Void,
    Class(String),
    Array(Box<VarType>),
    // non-JVM standard
    Tuple(Vec<VarType>),
}

impl VarType {
    pub fn slot(&self) -> usize {
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

    pub fn size(&self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Short | Self::Char => 2,
            Self::Int | Self::Boolean | Self::Float | Self::Class(_) | Self::Array(_) => 4,
            Self::Double | Self::Long => 8,
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

    pub fn descriptor(&self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for VarType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VarType::Boolean => write!(f, "Z"),
            VarType::Byte => write!(f, "B"),
            VarType::Char => write!(f, "C"),
            VarType::Short => write!(f, "S"),
            VarType::Int => write!(f, "I"),
            VarType::Long => write!(f, "J"),
            VarType::Float => write!(f, "F"),
            VarType::Double => write!(f, "D"),
            VarType::Void => write!(f, "V"),
            VarType::Class(s) => write!(f, "L{};", s),
            VarType::Array(t) => write!(f, "[{}", t),
            VarType::Tuple(vs) => write!(
                f,
                "({}",
                vs.iter().map(|t| format!("{}", t)).collect::<String>()
            ),
        }
    }
}

pub fn fn_descriptor(ret_ty: &VarType, ps: &Vec<VarType>) -> String {
    format!(
        "({}){}",
        ps.iter().map(|t| format!("{}", t)).collect::<String>(),
        ret_ty
    )
}
