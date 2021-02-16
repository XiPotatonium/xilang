use std::fmt;

#[derive(Clone, Eq)]
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

impl PartialEq for VarType {
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

pub struct MethodType {
    pub class_name: String,
    pub method_name: String,
}

pub enum XirType {
    RVal(VarType),
    // class full name, method name
    Method(String, String),
    // class full name, field name
    Field(String, String),
    // class full name
    Class(String),
    // module full name
    Module(String),
    // offset
    Local(u16),
}

impl XirType {
    pub fn expect_rval(self) -> VarType {
        match self {
            Self::RVal(ret) => ret,
            _ => panic!("Expect XirType::VarType"),
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
