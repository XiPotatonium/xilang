use ir::flags::ClassFlags;

use super::super::util::{IItemPath, ItemPathBuf};
use super::disp::BoxASTVecWrapper;
use super::ASTGenericParamDecl;
use super::AST;

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
    pub generic_params: Vec<ASTGenericParamDecl>,
    pub impls: Vec<ItemPathBuf>,
    pub fields: Vec<Box<AST>>,
    pub methods: Vec<Box<AST>>,
    /// AST::Block
    pub cctor: Box<AST>,
}

impl ASTStruct {
    pub fn ast_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\"name\":\"(struct){}", self.name,)?;
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
            "\",\"flags\":\"{}\",\"cus-attrib\":{},\"impls\":[{}],\"fields\":{},\"cctor\":{},\"methods\":{}}}",
            self.flags,
            BoxASTVecWrapper(&self.custom_attribs),
            self.impls.iter().map(|p| format!("\"{}\"", p.as_str())).collect::<Vec<String>>().join(","),
            BoxASTVecWrapper(&self.fields),
            self.cctor,
            BoxASTVecWrapper(&self.methods),
        )
    }
}
