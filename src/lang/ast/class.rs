use xir::attrib::TypeAttrib;
use xir::util::path::{IModPath, ModPath};

use super::disp::ASTChildrenWrapper;
use super::{ASTCtor, AST};

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
    /// ASTCtor
    pub ctors: Vec<Box<ASTCtor>>,
}

impl fmt::Display for ASTClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(class){}\",\"attrib\":\"{}\",\"cus-attrib\":{},\"extends-or-impls\":[{}],\"fields\":{},\"cctor\":{},\"ctors\":{},\"methods\":{}}}",
            self.name,
            self.attrib,
            ASTChildrenWrapper(&self.custom_attribs),
            self.extends_or_impls.iter().map(|p| format!("\"{}\"", p.as_str())).collect::<Vec<String>>().join(","),
            ASTChildrenWrapper(&self.fields),
            self.cctor,
            ASTChildrenWrapper(&self.ctors),
            ASTChildrenWrapper(&self.methods),
        )
    }
}
