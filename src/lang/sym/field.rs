use ir::flags::{FieldFlag, FieldFlags};

use super::{RValType, Struct};

use std::fmt;
use std::ptr::NonNull;

pub struct Field {
    pub parent: NonNull<Struct>,

    pub name: String,

    pub attrib: FieldFlags,
    pub ty: RValType,

    /// index into field tbl
    pub idx: u32,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.parent.as_ref() })?;
        if self.attrib.is(FieldFlag::Static) {
            write!(f, "::")?;
        } else {
            write!(f, ".")?;
        }
        write!(f, "{}: {}", self.name, self.ty)
    }
}
