use core::util::{IItemPath, ItemPathBuf};

use std::fmt;

#[derive(Clone)]
pub enum Type {
    Bool,
    Char,
    I32,
    F64,
    ISize,
    USize,
    Str,
    Tuple(Vec<Box<Type>>),
    /// type
    Arr(Box<Type>),
    UsrType(ItemPathBuf),
    /// void or undetermined
    None,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "(type)bool"),
            Type::Char => write!(f, "(type)char"),
            Type::I32 => write!(f, "(type)i32"),
            Type::F64 => write!(f, "(type)f64"),
            Type::ISize => write!(f, "(type)isize"),
            Type::USize => write!(f, "(type)usize"),
            Type::Str => write!(f, "(type)string"),
            Type::Tuple(_) => unimplemented!(),
            Type::Arr(dtype) => write!(f, "(type){}[]", dtype),
            Type::UsrType(names) => write!(f, "(type){}", names.as_str()),
            Type::None => write!(f, "(type)none"),
        }
    }
}
