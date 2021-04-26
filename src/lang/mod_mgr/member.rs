use xir::attrib::{
    FieldAttrib, FieldAttribFlag, MethodAttrib, MethodAttribFlag, MethodImplAttrib, ParamAttrib,
};

use super::super::gen::RValType;
use super::Class;

use std::fmt;

pub struct Method {
    pub parent: *const Class,

    pub name: String,

    pub ret: RValType,
    /// self is not included
    pub ps: Vec<Param>,
    pub attrib: MethodAttrib,
    pub impl_flag: MethodImplAttrib,

    /// index into methoddef tbl
    pub idx: u32,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.parent.as_ref().unwrap() })?;
        if self.attrib.is(MethodAttribFlag::Static) {
            write!(f, "::")?;
        } else {
            write!(f, ".")?;
        }
        write!(f, "{}: (", self.name)?;
        for (i, p) in self.ps.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", p.id, p.ty)?;
        }
        write!(f, ")")?;
        if let RValType::Void = self.ret {
        } else {
            write!(f, " -> {}", self.ret)?;
        }
        Ok(())
    }
}

pub struct Param {
    pub id: String,
    pub attrib: ParamAttrib,
    pub ty: RValType,
}

pub struct Field {
    pub parent: *const Class,

    pub name: String,

    pub attrib: FieldAttrib,
    pub ty: RValType,

    /// index into field tbl
    pub idx: u32,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.parent.as_ref().unwrap() })?;
        if self.attrib.is(FieldAttribFlag::Static) {
            write!(f, "::")?;
        } else {
            write!(f, ".")?;
        }
        write!(f, "{}: {}", self.name, self.ty)
    }
}
