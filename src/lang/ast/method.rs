use xir::attrib::MethodAttrib;

use super::disp::ASTChildrenWrapper;
use super::AST;

use std::convert::TryFrom;
use std::fmt;

const ASTMETHOD_ATTRIB_OVERRIDE_FLAG: u16 = 0x0001;

pub enum ASTMethodAttribFlag {
    Override,
}

impl From<ASTMethodAttribFlag> for u16 {
    fn from(value: ASTMethodAttribFlag) -> Self {
        match value {
            ASTMethodAttribFlag::Override => ASTMETHOD_ATTRIB_OVERRIDE_FLAG,
        }
    }
}

impl TryFrom<u16> for ASTMethodAttribFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            ASTMETHOD_ATTRIB_OVERRIDE_FLAG => Ok(ASTMethodAttribFlag::Override),
            _ => Err("Invalid value for ASTMethodAttribFlag"),
        }
    }
}

pub struct ASTMethodAttrib {
    attrib: u16,
}

impl ASTMethodAttrib {
    pub fn from(attrib: u16) -> ASTMethodAttrib {
        ASTMethodAttrib { attrib }
    }

    pub fn is(&self, flag: ASTMethodAttribFlag) -> bool {
        self.attrib & u16::from(flag) != 0
    }

    pub fn set(&mut self, flag: ASTMethodAttribFlag) {
        self.attrib |= u16::from(flag);
    }
}

impl Default for ASTMethodAttrib {
    fn default() -> Self {
        ASTMethodAttrib { attrib: 0 }
    }
}

pub struct ASTMethod {
    pub name: String,
    pub attrib: MethodAttrib,
    /// ast attrib are some special built-in attribute that only work at compile time
    pub ast_attrib: ASTMethodAttrib,
    pub custom_attribs: Vec<Box<AST>>,
    pub ret: Box<AST>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
                f,
                "{{\"name\":\"(method){}\",\"attrib\":\"{}\",\"custom-attribs\":{},\"ret\":{},\"ps\":{},\"body\":{}}}",
                self.name,
                self.attrib,
                ASTChildrenWrapper(&self.custom_attribs),
                self.ret,
                ASTChildrenWrapper(&self.ps),
                self.body,
            )
    }
}

pub struct ASTCtor {
    pub attrib: MethodAttrib,
    pub custom_attribs: Vec<Box<AST>>,
    pub base_args: Option<Vec<Box<AST>>>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTCtor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(.ctor)\",\"attrib\":\"{}\",\"custom-attribs\":{},\"base-args\":",
            self.attrib,
            ASTChildrenWrapper(&self.custom_attribs),
        )?;
        if let Some(args) = &self.base_args {
            write!(f, "{}", ASTChildrenWrapper(args))?;
        } else {
            write!(f, "[]")?;
        }
        write!(
            f,
            ",\"ps\":{},\"body\":{}}}",
            ASTChildrenWrapper(&self.ps),
            self.body,
        )
    }
}
