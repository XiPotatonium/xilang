use xir::util::path::{IModPath, ModPath};

use super::disp::ASTVecWrapper;
use super::AST;

use std::fmt;

pub enum ASTType {
    Bool,
    Char,
    I32,
    F64,
    String,
    Tuple(Vec<Box<ASTType>>),
    /// type, dim
    Arr(Box<ASTType>, Box<AST>),
    /// class names
    Class(ModPath),
    /// void or undetermined
    None,
}

impl fmt::Display for ASTType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ASTType::Bool => write!(f, "{{\"name\":\"(type)bool\"}}"),
            ASTType::Char => write!(f, "{{\"name\":\"(type)char\"}}"),
            ASTType::I32 => write!(f, "{{\"name\":\"(type)i32\"}}"),
            ASTType::F64 => write!(f, "{{\"name\":\"(type)f64\"}}"),
            ASTType::String => write!(f, "{{\"name\":\"(type)string\"}}"),
            ASTType::Tuple(v) => write!(
                f,
                "{{\"name\":\"(type)tuple\",\"children\":{}}}",
                ASTVecWrapper(v)
            ),
            ASTType::Arr(dtype, dim) => write!(
                f,
                "{{\"name\":\"(type)array\",\"dtype\":{},\"dim\":{}}}",
                dtype, dim
            ),
            ASTType::Class(names) => write!(f, "{{\"name\":\"(type){}\"}}", names.as_str()),
            ASTType::None => write!(f, "{{\"name\":\"(type)none\"}}"),
        }
    }
}
