use core::flags::FuncFlags;
use core::util::{IItemPath, ItemPathBuf};

use super::super::ast::AST;
use super::{RValType, Symbol, TypeLinkContext};

use std::fmt;

pub struct Func {
    pub parent: Symbol,
    pub path: ItemPathBuf,

    pub ret: RValType,
    /// self is not included
    pub ps: Vec<Param>,
    pub flags: FuncFlags,

    pub body: Box<AST>,
}

impl Func {
    pub fn fullname(&self) -> &str {
        self.path.as_str()
    }

    pub fn name(&self) -> &str {
        self.path.get_self().unwrap()
    }
}

impl fmt::Display for Func {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fullname())
    }
}

pub struct Param {
    pub name: String,
    pub ty: RValType,
}

impl AsRef<RValType> for Param {
    fn as_ref(&self) -> &RValType {
        &self.ty
    }
}
