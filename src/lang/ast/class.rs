use xir::attrib::TypeAttrib;
use xir::util::path::{IModPath, ModPath};

use super::disp::BoxASTVecWrapper;
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
    /// AST::Ctor
    pub ctors: Vec<Box<AST>>,
}

impl ASTClass {
    pub fn ast_fmt(&self, f: &mut fmt::Formatter<'_>, is_struct: bool) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"({}){}\",\"attrib\":\"{}\",\"cus-attrib\":{},\"extends-or-impls\":[{}],\"fields\":{},\"cctor\":{},\"ctors\":{},\"methods\":{}}}",
            if is_struct { "struct" } else { "class" },
            self.name,
            self.attrib,
            BoxASTVecWrapper(&self.custom_attribs),
            self.extends_or_impls.iter().map(|p| format!("\"{}\"", p.as_str())).collect::<Vec<String>>().join(","),
            BoxASTVecWrapper(&self.fields),
            self.cctor,
            BoxASTVecWrapper(&self.ctors),
            BoxASTVecWrapper(&self.methods),
        )
    }
}
