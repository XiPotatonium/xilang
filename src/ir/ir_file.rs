use super::inst::Inst;

pub const MAJOR_VERSION: u16 = 1;
pub const MINOR_VERSION: u16 = 0;

#[derive(Debug)]
pub struct IrFile {
    pub minor_version: u16,
    pub major_version: u16,

    /// assert_eq!(mod_tbl.len(), 1)
    pub mod_tbl: Vec<IrMod>,
    pub modref_tbl: Vec<IrModRef>,

    pub type_tbl: Vec<IrType>,
    pub typeref_tbl: Vec<IrTypeRef>,

    pub field_tbl: Vec<IrField>,
    pub method_tbl: Vec<IrMethod>,

    pub memberref_tbl: Vec<IrMemberRef>,

    pub str_heap: Vec<String>,
    pub codes: Vec<Vec<Inst>>,
}

#[derive(Debug)]
pub struct IrMod {
    /// index into utf8 heap
    pub name: u32,

    /// index of codes
    pub entrypoint: u32,
}

#[derive(Debug)]
pub struct IrModRef {
    pub name: u32,
}

#[derive(Debug)]
pub struct IrType {
    pub name: u32,
    pub flag: u32,

    pub fields: u32,
    pub methods: u32,
}

#[derive(Debug)]
pub struct IrTypeRef {
    pub parent: u32,
    pub name: u32,
}

#[derive(Debug)]
pub struct IrField {
    pub name: u32,
    pub descriptor: u32,

    pub flag: u16,
}

#[derive(Debug)]
pub struct IrMethod {
    pub name: u32,
    pub descriptor: u32,

    pub flag: u16,
    pub locals: u16,
}

#[derive(Debug)]
pub struct IrMemberRef {
    pub parent: u32,
    pub name: u32,
    pub descriptor: u32,
}

pub const TBL_TAG_MASK: u32 = 0xFF << 24;
pub const TBL_MOD_TAG: u32 = 0x00 << 24;
pub const TBL_MODREF_TAG: u32 = 0x1A << 24;
pub const TBL_TYPE_TAG: u32 = 0x02 << 24;
pub const TBL_TYPEREF_TAG: u32 = 0x01 << 24;
pub const TBL_FIELD_TAG: u32 = 0x04 << 24;
pub const TBL_METHOD_TAG: u32 = 0x06 << 24;
pub const TBL_MEMBERREF_TAG: u32 = 0x0A << 24;

pub enum TblValue<'f> {
    Mod(&'f IrMod),
    ModRef(&'f IrModRef),
    Type(&'f IrType),
    TypeRef(&'f IrTypeRef),
    Field(&'f IrField),
    Method(&'f IrMethod),
    MemberRef(&'f IrMemberRef),
    None,
}

impl IrFile {
    pub fn new() -> IrFile {
        IrFile {
            minor_version: MINOR_VERSION,
            major_version: MAJOR_VERSION,

            mod_tbl: vec![],
            modref_tbl: vec![],

            type_tbl: vec![],
            typeref_tbl: vec![],

            field_tbl: vec![],
            method_tbl: vec![],
            memberref_tbl: vec![],

            str_heap: vec![],
            codes: vec![],
        }
    }

    pub fn mod_name(&self) -> Option<&str> {
        if self.mod_tbl.is_empty() {
            None
        } else {
            Some(&self.str_heap[self.mod_tbl[0].name as usize])
        }
    }

    pub fn get_tbl_entry(&self, idx: u32) -> TblValue {
        let tag = idx & TBL_TAG_MASK;
        let idx = (idx & !TBL_TAG_MASK) as usize;
        let idx = if idx == 0 {
            return TblValue::None;
        } else {
            idx - 1
        };

        match tag {
            TBL_MOD_TAG => TblValue::Mod(&self.mod_tbl[idx]),
            TBL_MODREF_TAG => TblValue::ModRef(&self.modref_tbl[idx]),
            TBL_TYPE_TAG => TblValue::Type(&self.type_tbl[idx]),
            TBL_TYPEREF_TAG => TblValue::TypeRef(&self.typeref_tbl[idx]),
            TBL_METHOD_TAG => TblValue::Method(&self.method_tbl[idx]),
            TBL_FIELD_TAG => TblValue::Field(&self.field_tbl[idx]),
            TBL_MEMBERREF_TAG => TblValue::MemberRef(&self.memberref_tbl[idx]),
            _ => TblValue::None,
        }
    }

    pub fn get_str(&self, idx: u32) -> &str {
        &self.str_heap[idx as usize]
    }
}
