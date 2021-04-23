use xir::attrib::MethodAttrib;

use super::disp::ASTChildrenWrapper;
use super::AST;

use std::fmt;

pub struct ASTMethod {
    pub name: String,
    pub attrib: MethodAttrib,
    pub custom_attribs: Vec<Box<AST>>,
    pub ty: Box<AST>,
    pub ps: Vec<Box<AST>>,
    pub body: Box<AST>,
}

impl fmt::Display for ASTMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
                f,
                "{{\"name\":\"(method){}\",\"attrib\":\"{}\",\"attr\":{},\"type\":{},\"ps\":{},\"body\":{}}}",
                self.name,
                self.attrib,
                ASTChildrenWrapper(&self.custom_attribs),
                self.ty,
                ASTChildrenWrapper(&self.ps),
                self.body,
            )
    }
}
