use xir::attrib::TypeAttrib;
use xir::util::path::{IModPath, ModPath};

use super::disp::ASTChildrenWrapper;
use super::AST;

use std::fmt;

pub struct ASTClass {
    pub name: String,
    pub attrib: TypeAttrib,
    /// AST::CustomAttrib
    pub custom_attribs: Vec<Box<AST>>,
    pub extends_or_impls: Vec<ModPath>,
    pub fields: Vec<Box<AST>>,
    pub methods: Vec<Box<AST>>,
    pub cctor: Box<AST>,
    pub ctors: Vec<Box<AST>>,
}

impl fmt::Display for ASTClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(class){}\",\"attrib\":\"{}\",\"cus-attrib\":{},\"extends-or-impls\":[{}],\"fields\":{},\"cctor\":{},\"methods\":{}}}",
            self.name,
            self.attrib,
            ASTChildrenWrapper(&self.custom_attribs),
            self.extends_or_impls.iter().map(|p| format!("\"{}\"", p.as_str())).collect::<Vec<String>>().join(","),
            ASTChildrenWrapper(&self.fields),
            self.cctor,
            ASTChildrenWrapper(&self.methods),
        )
    }
}
