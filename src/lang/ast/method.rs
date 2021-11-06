use ir::flags::MethodFlags;

use super::disp::BoxASTVecWrapper;
use super::{ASTGenericParamDecl, ASTType, AST};

use std::convert::TryFrom;
use std::fmt;

const ASTMETHOD_OVERRIDE: u16 = 0x0001;

pub enum ASTMethodFlag {
    Override,
}

impl From<ASTMethodFlag> for u16 {
    fn from(value: ASTMethodFlag) -> Self {
        match value {
            ASTMethodFlag::Override => ASTMETHOD_OVERRIDE,
        }
    }
}

impl TryFrom<u16> for ASTMethodFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            ASTMETHOD_OVERRIDE => Ok(ASTMethodFlag::Override),
            _ => Err("Invalid value for ASTMethodAttribFlag"),
        }
    }
}

pub struct ASTMethodFlags {
    attrib: u16,
}

impl ASTMethodFlags {
    pub fn is(&self, flag: ASTMethodFlag) -> bool {
        self.attrib & u16::from(flag) != 0
    }

    pub fn set(&mut self, flag: ASTMethodFlag) {
        self.attrib |= u16::from(flag);
    }
}

impl Default for ASTMethodFlags {
    fn default() -> Self {
        ASTMethodFlags { attrib: 0 }
    }
}

pub struct ASTMethod {
    pub name: String,
    pub flags: MethodFlags,
    /// ast attrib are some special built-in attribute that only work at compile time
    pub ast_flags: ASTMethodFlags,
    pub custom_attribs: Vec<Box<AST>>,
    pub generic_params: Vec<ASTGenericParamDecl>,
    pub ret: Box<ASTType>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\"name\":\"(method){}", self.name)?;
        if !self.generic_params.is_empty() {
            write!(f, "<")?;
            for (i, generic_p) in self.generic_params.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", generic_p)?;
            }
            write!(f, ">")?;
        }
        write!(
            f,
            "\",\"flags\":\"{}\",\"custom-attribs\":{},\"ret\":\"{}\",\"ps\":{},\"body\":{}}}",
            self.flags,
            BoxASTVecWrapper(&self.custom_attribs),
            self.ret,
            BoxASTVecWrapper(&self.ps),
            self.body,
        )
    }
}

pub struct ASTCtor {
    pub flags: MethodFlags,
    // pub custom_attribs: Vec<Box<AST>>,
    pub generic_params: Vec<ASTGenericParamDecl>,
    pub base_args: Option<Vec<Box<AST>>>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTCtor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\"name\":\"(.ctor)")?;
        if !self.generic_params.is_empty() {
            write!(f, "<")?;
            for (i, generic_p) in self.generic_params.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", generic_p)?;
            }
            write!(f, ">")?;
        }
        write!(f, "\",\"flags\":\"{}\",\"base-args\":", self.flags,)?;
        if let Some(args) = &self.base_args {
            write!(f, "{}", BoxASTVecWrapper(args))?;
        } else {
            write!(f, "[]")?;
        }
        write!(
            f,
            ",\"ps\":{},\"body\":{}}}",
            BoxASTVecWrapper(&self.ps),
            self.body,
        )
    }
}
