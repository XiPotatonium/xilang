use super::disp::BoxASTVecWrapper;
use super::AST;
use core::flags::ClassFlags;
use core::util::{IItemPath, ItemPathBuf};

use std::fmt;

pub struct ASTFieldInit {
    pub field: String,
    /// expr
    pub value: Box<AST>,
}

impl fmt::Display for ASTFieldInit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(field-init)\",\"field\":\"{}\",\"value\":{}}}",
            self.field, self.value
        )
    }
}

pub struct ASTClass {
    pub name: String,
    pub flags: ClassFlags,
    /// AST::CustomAttrib
    pub custom_attribs: Vec<Box<AST>>,
    pub impls: Vec<ItemPathBuf>,
    pub fields: Vec<Box<AST>>,
    pub methods: Vec<Box<AST>>,
}

impl ASTClass {
    pub fn ast_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(class){}\",\"flags\":\"{}\",\"custom-attrib\":{},\"impls\":[{}],\"fields\":{},\"methods\":{}}}",
            self.name,
            self.flags,
            BoxASTVecWrapper(&self.custom_attribs),
            self.impls.iter().map(|p| format!("\"{}\"", p.as_str())).collect::<Vec<String>>().join(","),
            BoxASTVecWrapper(&self.fields),
            BoxASTVecWrapper(&self.methods),
        )
    }
}
