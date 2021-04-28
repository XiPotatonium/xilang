use xir::attrib::{
    FieldAttrib, FieldAttribFlag, MethodAttrib, MethodAttribFlag, MethodImplAttrib, ParamAttrib,
};

use super::super::ast::AST;
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

    /// None for external method (IL or dll) or automatically generated methods (default ctor)
    ///
    /// Some(AST::Body) for cctor
    ///
    /// Some(AST::Ctor) for ctor
    ///
    /// Some(AST::Method) for normal method
    pub ast: Option<*const AST>,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.parent.as_ref().unwrap() })?;
        if self.attrib.is(MethodAttribFlag::Static) {
            write!(f, "::")?;
        } else {
            write!(f, ".")?;
        }
        write!(f, "{}", self.name)
    }
}

pub struct Param {
    pub id: String,
    pub attrib: ParamAttrib,
    pub ty: RValType,
}

impl AsRef<RValType> for Param {
    fn as_ref(&self) -> &RValType {
        &self.ty
    }
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
