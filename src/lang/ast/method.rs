use ir::flags::MethodFlags;

use super::disp::BoxASTVecWrapper;
use super::{ASTGenericParamDecl, ASTType, AST};

use std::fmt;

pub struct ASTMethod {
    pub name: String,
    pub flags: MethodFlags,
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
