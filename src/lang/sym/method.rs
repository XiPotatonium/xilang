use ir::flags::{MethodFlag, MethodFlags};

use super::{RValType, Struct};

use std::fmt;
use std::ptr::NonNull;

/// TODO: rename
pub fn method_descriptor1(ps: &Vec<Param>, ret: &RValType) -> String {
    let mut buf = String::from("(");
    for p in ps.iter() {
        buf.push_str(&p.ty.descriptor());
    }
    buf.push(')');
    buf.push_str(&ret.descriptor());
    buf
}

pub fn method_descriptor2(ps_ty: &Vec<RValType>, ret: &RValType) -> String {
    let mut buf = String::from("(");
    for p in ps_ty.iter() {
        buf.push_str(&p.descriptor());
    }
    buf.push(')');
    buf.push_str(&ret.descriptor());
    buf
}

pub struct Method {
    pub parent: NonNull<Struct>,

    pub name: String,

    pub ret: RValType,
    /// self is not included
    pub ps: Vec<Param>,
    pub flags: MethodFlags,

    /// index into methoddef tbl
    pub idx: usize,
}

impl Method {
    pub fn sig_match(&self, m1: &Method) -> bool {
        if self.name != m1.name
            || self.ps.len() != m1.ps.len()
            || self.flags.is(MethodFlag::Static) != m1.flags.is(MethodFlag::Static)
        {
            return false;
        }

        for (p, p1) in self.ps.iter().zip(m1.ps.iter()) {
            if p.ty != p1.ty {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.parent.as_ref() })?;
        if self.flags.is(MethodFlag::Static) {
            write!(f, "::")?;
        } else {
            write!(f, ".")?;
        }
        write!(f, "{}", self.name)
    }
}

pub struct Param {
    pub id: String,
    pub ty: RValType,
}

impl AsRef<RValType> for Param {
    fn as_ref(&self) -> &RValType {
        &self.ty
    }
}
