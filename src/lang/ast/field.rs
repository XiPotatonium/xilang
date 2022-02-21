use super::ASTType;
use core::flags::FieldFlags;

use std::fmt;

pub struct ASTField {
    pub name: String,
    pub flags: FieldFlags,
    pub ty: Box<ASTType>,
}

impl fmt::Display for ASTField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{\"name\":\"(field){}\",\"flags\":\"{}\",\"type\":\"{}\"}}",
            self.name, self.flags, self.ty
        )
    }
}
