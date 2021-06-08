use super::super::util::{IItemPath, ItemPathBuf};
use super::ASTType;

use std::fmt;

pub struct ASTGenericParamDecl {
    pub id: String,
    pub constraints: Vec<ItemPathBuf>,
}

pub struct ASTIdWithGenericParam {
    pub id: String,
    pub generic_params: Vec<Box<ASTType>>,
}

impl fmt::Display for ASTGenericParamDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)?;
        if !self.constraints.is_empty() {
            write!(f, ":")?;
            for (i, constraint) in self.constraints.iter().enumerate() {
                if i != 0 {
                    write!(f, " +")?;
                }
                write!(f, " {}", constraint.as_str())?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for ASTIdWithGenericParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)?;
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

        Ok(())
    }
}
