use super::disp::BoxASTVecWrapper;
use super::{ASTType, AST};
use core::flags::FuncFlags;

use std::fmt;

pub struct ASTFunc {
    pub name: String,
    pub flags: FuncFlags,
    pub custom_attribs: Vec<Box<AST>>,
    pub ret: Box<ASTType>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(fn){}\",\"flags\":\"{}\",\"custom-attribs\":{},\"ret\":\"{}\",\"params\":{},\"body\":{}}}",
            self.name,
            self.flags,
            BoxASTVecWrapper(&self.custom_attribs),
            self.ret,
            BoxASTVecWrapper(&self.ps),
            self.body,
        )
    }
}
