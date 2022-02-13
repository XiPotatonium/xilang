use super::disp::BoxASTVecWrapper;
use super::{ASTType, AST};
use core::flags::{FieldFlags, MethodFlags};

use std::fmt;

pub struct ASTField {
    pub name: String,
    pub flags: FieldFlags,
    pub ty: Box<ASTType>,
}

impl fmt::Display for ASTField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(field){}\",\"flag\":\"{}\",\"type\":\"{}\"}}",
            self.name, self.flags, self.ty
        )
    }
}

pub struct ASTMethod {
    pub name: String,
    pub flags: MethodFlags,
    pub custom_attribs: Vec<Box<AST>>,
    pub ret: Box<ASTType>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(method){}\",\"flags\":\"{}\",\"custom-attribs\":{},\"ret\":\"{}\",\"ps\":{},\"body\":{}}}",
            self.name,
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
    pub base_args: Option<Vec<Box<AST>>>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTCtor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(.ctor)\",\"flags\":\"{}\",\"base-args\":",
            self.flags,
        )?;
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
