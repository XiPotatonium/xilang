use super::super::util::{IItemPath, ItemPathBuf};
use super::disp::BoxASTVecWrapper;
use super::{ASTField, ASTMethod, AST};
use core::flags::ClassFlags;

use std::fmt;

pub struct ASTStructFieldInit {
    pub field: String,
    /// expr
    pub value: Box<AST>,
}

impl fmt::Display for ASTStructFieldInit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(field-init)\",\"field\":\"{}\",\"value\":{}}}",
            self.field, self.value
        )
    }
}

pub struct ASTStruct {
    pub name: String,
    pub flags: ClassFlags,
    /// AST::CustomAttrib
    pub custom_attribs: Vec<Box<AST>>,
    pub impls: Vec<ItemPathBuf>,
    pub fields: Vec<Box<AST>>,
    pub methods: Vec<Box<AST>>,
    /// AST::Block
    pub cctor: Box<AST>,
}

impl ASTStruct {
    pub fn ast_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(struct){}\",\"flags\":\"{}\",\"cus-attrib\":{},\"impls\":[{}],\"fields\":{},\"cctor\":{},\"methods\":{}}}",
            self.name,
            self.flags,
            BoxASTVecWrapper(&self.custom_attribs),
            self.impls.iter().map(|p| format!("\"{}\"", p.as_str())).collect::<Vec<String>>().join(","),
            BoxASTVecWrapper(&self.fields),
            self.cctor,
            BoxASTVecWrapper(&self.methods),
        )
    }
}
