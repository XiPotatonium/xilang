use super::blob::IrSig;
use super::code::CorILMethod;
use super::member::{IrField, IrImplMap, IrMemberRef, IrMethodDef};
use super::module::{IrMod, IrModRef};
use super::param::IrParam;
use super::stand_alone_sig::IrStandAloneSig;
use super::ty::{IrTypeDef, IrTypeRef};

pub const MAJOR_VERSION: u16 = 0;
pub const MINOR_VERSION: u16 = 2;

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
    pub method_tbl: Vec<IrMethodDef>,

    pub memberref_tbl: Vec<IrMemberRef>,

    pub implmap_tbl: Vec<IrImplMap>,

    pub param_tbl: Vec<IrParam>,
    pub stand_alone_sig_tbl: Vec<IrStandAloneSig>,

    /// index starts from 0
    pub str_heap: Vec<String>,
    /// index starts from 0
    pub usr_str_heap: Vec<String>,
    pub blob_heap: Vec<IrSig>,

    /// None CLR standard, index starts from 1
    pub codes: Vec<CorILMethod>,
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
            param_tbl: vec![],
            stand_alone_sig_tbl: vec![],

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
