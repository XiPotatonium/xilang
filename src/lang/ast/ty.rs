use super::super::util::{IItemPath, ItemPathBuf};

use std::fmt;

#[derive(Clone)]
pub enum ASTType {
    Bool,
    Char,
    I32,
    F64,
    String,
    Tuple(Vec<Box<ASTType>>),
    /// type
    Arr(Box<ASTType>),
    UsrType(ItemPathBuf),
    /// void or undetermined
    None,
}

impl fmt::Display for ASTType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ASTType::Bool => write!(f, "(type)bool"),
            ASTType::Char => write!(f, "(type)char"),
            ASTType::I32 => write!(f, "(type)i32"),
            ASTType::F64 => write!(f, "(type)f64"),
            ASTType::String => write!(f, "(type)string"),
            ASTType::Tuple(_) => unimplemented!(),
            ASTType::Arr(dtype) => write!(f, "(type){}[]", dtype),
            ASTType::UsrType(names) => write!(f, "(type){}", names.as_str()),
            ASTType::None => write!(f, "(type)none"),
        }
    }
}
