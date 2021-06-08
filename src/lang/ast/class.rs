use xir::attrib::TypeAttrib;

use super::super::util::{IItemPath, ItemPathBuf};
use super::disp::BoxASTVecWrapper;
use super::ASTGenericParamDecl;
use super::AST;

use std::fmt;

pub struct ASTClass {
    pub name: String,
    pub attrib: TypeAttrib,
    /// AST::CustomAttrib
    pub custom_attribs: Vec<Box<AST>>,
    pub generic_params: Vec<ASTGenericParamDecl>,
    pub extends_or_impls: Vec<ItemPathBuf>,
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
            "{{\"name\":\"({}){}",
            if is_struct { "struct" } else { "class" },
            self.name,
        )?;
        if !self.generic_params.is_empty() {
            write!(f, "<")?;
            for (i, generic_p) in self.generic_params.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", generic_p)?;
            }
        }
        write!(
            f,
            "\",\"attrib\":\"{}\",\"cus-attrib\":{},\"extends-or-impls\":[{}],\"fields\":{},\"cctor\":{},\"ctors\":{},\"methods\":{}}}",
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
