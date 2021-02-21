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

    /// type tbl in CLR
    pub class_tbl: Vec<IrClass>,
    /// type ref tbl in CLR
    pub classref_tbl: Vec<IrClassRef>,

    pub field_tbl: Vec<IrField>,
    pub method_tbl: Vec<IrMethod>,

    pub memberref_tbl: Vec<IrMemberRef>,

    pub str_heap: Vec<String>,
    /// none CLR standard
    pub blob_heap: Vec<IrBlob>,
    pub codes: Vec<Vec<Inst>>,
}

#[derive(Debug)]
pub struct IrMod {
    /// index into str heap
    pub name: u32,

    /// index of codes
    pub entrypoint: u32,
}

#[derive(Debug)]
pub struct IrModRef {
    /// index into str heap
    pub name: u32,
}

#[derive(Debug)]
pub struct IrClass {
    /// index into str heap
    pub name: u32,
    /// IrTypeFlag
    pub flag: u32,

    /// index into field tbl
    pub fields: u32,
    /// into into method tbl
    pub methods: u32,
}

#[derive(Debug)]
pub struct IrClassRef {
    /// index into mod/modref tbl
    pub parent: u32,
    /// index into str heap
    pub name: u32,
}

#[derive(Debug)]
pub struct IrField {
    /// index into str heap
    pub name: u32,
    /// index into blob heap
    pub signature: u32,

    /// IrFieldFlag
    pub flag: u16,
}

#[derive(Debug)]
pub struct IrMethod {
    /// index into str heap
    pub name: u32,
    /// index into blob heap
    pub signature: u32,

    /// IrMethodFlag
    pub flag: u16,
    /// local size
    pub locals: u16,
}

#[derive(Debug)]
pub struct IrMemberRef {
    /// index into mod/modref/type/typeref tbl
    pub parent: u32,
    /// index into str heap
    pub name: u32,
    /// index into blob heap
    pub signature: u32,
}

/// None CLR standard
#[derive(Debug)]
pub enum IrBlob {
    Void,
    Bool,
    Char,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    UNative,
    INative,
    F32,
    F64,
    Obj(u32),
    Func(Vec<u32>, u32),
    Array(u32),
}

pub const TBL_TAG_MASK: u32 = 0xFF << 24;
pub const TBL_MOD_TAG: u32 = 0x00 << 24;
pub const TBL_MODREF_TAG: u32 = 0x1A << 24;
pub const TBL_CLASS_TAG: u32 = 0x02 << 24;
pub const TBL_CLASSREF_TAG: u32 = 0x01 << 24;
pub const TBL_FIELD_TAG: u32 = 0x04 << 24;
pub const TBL_METHOD_TAG: u32 = 0x06 << 24;
pub const TBL_MEMBERREF_TAG: u32 = 0x0A << 24;

pub enum TblValue<'f> {
    Mod(&'f IrMod),
    ModRef(&'f IrModRef),
    Class(&'f IrClass),
    ClassRef(&'f IrClassRef),
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

            class_tbl: vec![],
            classref_tbl: vec![],

            field_tbl: vec![],
            method_tbl: vec![],
            memberref_tbl: vec![],

            str_heap: vec![],
            blob_heap: vec![],
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
            TBL_CLASS_TAG => TblValue::Class(&self.class_tbl[idx]),
            TBL_CLASSREF_TAG => TblValue::ClassRef(&self.classref_tbl[idx]),
            TBL_METHOD_TAG => TblValue::Method(&self.method_tbl[idx]),
            TBL_FIELD_TAG => TblValue::Field(&self.field_tbl[idx]),
            TBL_MEMBERREF_TAG => TblValue::MemberRef(&self.memberref_tbl[idx]),
            _ => TblValue::None,
        }
    }

    pub fn get_str(&self, idx: u32) -> &str {
        &self.str_heap[idx as usize]
    }

    pub fn get_blob_repr(&self, idx: u32) -> String {
        let blob = &self.blob_heap[idx as usize];
        match blob {
            IrBlob::Void => String::from("V"),
            IrBlob::Bool => String::from("Z"),
            IrBlob::Char => String::from("C"),
            IrBlob::U8 => String::from("B"),
            IrBlob::I8 => String::from("b"),
            IrBlob::U16 => String::from("S"),
            IrBlob::I16 => String::from("s"),
            IrBlob::U32 => String::from("I"),
            IrBlob::I32 => String::from("i"),
            IrBlob::U64 => String::from("L"),
            IrBlob::I64 => String::from("l"),
            IrBlob::UNative => String::from("N"),
            IrBlob::INative => String::from("n"),
            IrBlob::F32 => String::from("F"),
            IrBlob::F64 => String::from("f"),
            IrBlob::Obj(ty) => format!("O{};", self.get_tbl_entry_repr(*ty)),
            IrBlob::Func(ps, ret_ty) => format!(
                "({}){}",
                ps.iter()
                    .map(|p| self.get_blob_repr(*p))
                    .collect::<String>(),
                self.get_blob_repr(*ret_ty)
            ),
            IrBlob::Array(inner) => format!("[{}", self.get_blob_repr(*inner)),
        }
    }

    pub fn get_tbl_entry_repr(&self, idx: u32) -> String {
        match self.get_tbl_entry(idx) {
            TblValue::Mod(IrMod { name, .. }) => format!("{}", self.get_str(*name),),
            TblValue::ModRef(IrModRef { name }) => format!("{}", self.get_str(*name),),
            TblValue::Class(IrClass { name, .. }) => {
                format!("{}/{}", self.mod_name().unwrap(), self.get_str(*name))
            }
            TblValue::ClassRef(IrClassRef { parent, name }) => {
                format!(
                    "{}/{}",
                    self.get_tbl_entry_repr(*parent),
                    self.get_str(*name)
                )
            }
            TblValue::Field(IrField {
                name, signature, ..
            }) => {
                let self_idx = idx & !TBL_TAG_MASK;

                if self.class_tbl.is_empty() || self_idx < self.class_tbl[0].fields {
                    // field has no parent
                    format!(
                        "{}::{}: {}",
                        self.mod_name().unwrap(),
                        self.get_str(*name),
                        self.get_blob_repr(*signature)
                    )
                } else {
                    let mut ty_idx = 0;
                    while ty_idx < self.class_tbl.len() {
                        if self.class_tbl[ty_idx].fields > self_idx {
                            break;
                        }
                        ty_idx += 1;
                    }

                    format!(
                        "{}::{}: {}",
                        self.get_tbl_entry_repr(ty_idx as u32 | TBL_CLASS_TAG),
                        self.get_str(*name),
                        self.get_blob_repr(*signature)
                    )
                }
            }
            TblValue::Method(IrMethod {
                name, signature, ..
            }) => {
                let self_idx = idx & !TBL_TAG_MASK;

                if self.class_tbl.len() == 0 || self_idx < self.class_tbl[0].methods {
                    // method has no parent
                    format!(
                        "{}::{}: {}",
                        self.mod_name().unwrap(),
                        self.get_str(*name),
                        self.get_blob_repr(*signature)
                    )
                } else {
                    let mut ty_idx = 0;
                    while ty_idx < self.class_tbl.len() {
                        if self.class_tbl[ty_idx].methods > self_idx {
                            break;
                        }
                        ty_idx += 1;
                    }

                    format!(
                        "{}::{}: {}",
                        self.get_tbl_entry_repr(ty_idx as u32 | TBL_CLASS_TAG),
                        self.get_str(*name),
                        self.get_blob_repr(*signature)
                    )
                }
            }
            TblValue::MemberRef(IrMemberRef {
                parent,
                name,
                signature,
            }) => format!(
                "{}::{}: {}",
                self.get_tbl_entry_repr(*parent),
                self.get_str(*name),
                self.get_blob_repr(*signature)
            ),
            TblValue::None => String::new(),
        }
    }
}
