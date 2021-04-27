use xir::attrib::MethodAttrib;

use super::disp::ASTChildrenWrapper;
use super::AST;

use std::fmt;

pub struct ASTMethod {
    pub name: String,
    pub attrib: MethodAttrib,
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
    pub base_args: Vec<Box<AST>>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTCtor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
                f,
                "{{\"name\":\"(.ctor)\",\"attrib\":\"{}\",\"custom-attribs\":{},\"base-args\":{},\"ps\":{},\"body\":{}}}",
                self.attrib,
                ASTChildrenWrapper(&self.custom_attribs),
                ASTChildrenWrapper(&self.base_args),
                ASTChildrenWrapper(&self.ps),
                self.body,
            )
    }
}
