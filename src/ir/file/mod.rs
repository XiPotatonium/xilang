mod bc_serde;
mod code;
mod member;
mod text_serde;
mod ty;

use super::blob::Blob;
pub use code::CorILMethod;
pub use member::{to_memberref_parent, IrField, IrImplMap, IrMemberRef, IrMethod, MemberRefParent};
pub use ty::{get_typeref_parent, IrTypeDef, IrTypeRef, ResolutionScope};

use std::fmt;

pub const MAJOR_VERSION: u16 = 0;
pub const MINOR_VERSION: u16 = 1;

pub struct IrFile {
    pub major_version: u16,
    pub minor_version: u16,

    /// assert_eq!(mod_tbl.len(), 1)
    pub mod_tbl: Vec<IrMod>,
    pub modref_tbl: Vec<IrModRef>,

    /// type tbl in CLR
    pub typedef_tbl: Vec<IrTypeDef>,
    /// type ref tbl in CLR
    pub typeref_tbl: Vec<IrTypeRef>,

    pub field_tbl: Vec<IrField>,
    pub method_tbl: Vec<IrMethod>,

    pub memberref_tbl: Vec<IrMemberRef>,

    pub implmap_tbl: Vec<IrImplMap>,

    /// index starts from 0
    pub str_heap: Vec<String>,
    /// index starts from 0
    pub usr_str_heap: Vec<String>,
    pub blob_heap: Vec<Blob>,

    /// None CLR standard, index starts from 1
    pub codes: Vec<CorILMethod>,
}

pub trait IrFmt {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result;
}

pub struct IrMod {
    /// index into str heap
    pub name: u32,

    /// index of codes
    pub entrypoint: u32,
}

impl IrFmt for IrMod {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        write!(f, "{}", ctx.get_str(self.name))
    }
}

pub struct IrModRef {
    /// index into str heap
    pub name: u32,
}

impl IrFmt for IrModRef {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        write!(f, "{}", ctx.get_str(self.name))
    }
}

impl IrFile {
    pub fn new() -> IrFile {
        IrFile {
            major_version: MAJOR_VERSION,
            minor_version: MINOR_VERSION,

            mod_tbl: vec![],
            modref_tbl: vec![],

            typedef_tbl: vec![],
            typeref_tbl: vec![],

            field_tbl: vec![],
            method_tbl: vec![],
            memberref_tbl: vec![],

            implmap_tbl: vec![],

            str_heap: vec![],
            usr_str_heap: vec![],
            blob_heap: vec![],
            codes: vec![],
        }
    }

    pub fn mod_name(&self) -> &str {
        &self.str_heap[self.mod_tbl[0].name as usize]
    }

    pub fn get_str(&self, idx: u32) -> &str {
        &self.str_heap[idx as usize]
    }
}
