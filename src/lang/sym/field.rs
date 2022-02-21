use super::{Class, RValType};
use core::flags::FieldFlags;
use core::util::{IItemPath, ItemPathBuf};

use std::fmt;
use std::ptr::NonNull;

pub struct Field {
    pub parent: NonNull<Class>,
    pub path: ItemPathBuf,

    pub flags: FieldFlags,
    pub ty: RValType,
}

impl Field {
    pub fn fullname(&self) -> &str {
        self.path.as_str()
    }

    pub fn name(&self) -> &str {
        self.path.get_self().unwrap()
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fullname())
    }
}
