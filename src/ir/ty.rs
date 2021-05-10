use super::bc_serde::{IDeserializer, ISerializable};
use super::file::IrFile;
use super::text_serde::IrFmt;
use super::tok::TokTag;

use std::fmt;

pub struct TypeDef {
    /// IrTypeFlag
    pub flag: u32,
    /// index into str heap
    pub name: u32,

    /// index into TypedefOrRef table
    pub extends: u32,

    /// index into field tbl
    pub fields: u32,
    /// into into method tbl
    pub methods: u32,
}

impl TypeDef {
    pub fn fullname(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        write!(f, "{}/{}", ctx.mod_name(), ctx.get_str(self.name))
    }

    pub fn get_extends(&self) -> Option<(TypeDefOrRef, usize)> {
        if self.extends == 0 {
            None
        } else {
            let raw_idx = self.extends >> TYPEDEFORREF_TAG_SIZE;
            Some((
                match self.extends & TYPEDEFORREF_TAG_MASK {
                    0 => TypeDefOrRef::TypeDef,
                    1 => TypeDefOrRef::TypeRef,
                    3 => TypeDefOrRef::TypeSpec,
                    _ => unreachable!(),
                },
                raw_idx as usize - 1,
            ))
        }
    }

    pub fn set_extends(&mut self, raw_idx: u32, idx_tag: TypeDefOrRef) -> Option<u32> {
        let old_idx = if self.extends != 0 {
            Some(self.extends)
        } else {
            None
        };
        self.extends = (raw_idx << RESOLUTION_SCOPE_TAG_SIZE) | (idx_tag as u32);
        old_idx
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TypeRef {
    /// index into ResolutionScope tbl
    pub parent: u32,
    /// index into str heap
    pub name: u32,
}

impl TypeRef {
    pub fn get_parent(&self) -> (ResolutionScope, usize) {
        let raw_idx = self.parent >> RESOLUTION_SCOPE_TAG_SIZE;
        if raw_idx == 0 {
            panic!("Typeref has no parent");
        }
        (
            match self.parent & RESOLUTION_SCOPE_TAG_MASK {
                0 => ResolutionScope::Mod,
                1 => ResolutionScope::ModRef,
                2 => unimplemented!(),
                3 => ResolutionScope::TypeRef,
                _ => unreachable!(),
            },
            raw_idx as usize - 1,
        )
    }
}

impl TypeRef {
    pub fn fullname(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        let (parent_tag, parent_idx) = self.get_parent();
        write!(
            f,
            "{}/{}",
            match parent_tag {
                ResolutionScope::Mod => unimplemented!(),
                ResolutionScope::ModRef => ctx.get_str(ctx.modref_tbl[parent_idx].name),
                ResolutionScope::TypeRef => unimplemented!(),
            },
            ctx.get_str(self.name)
        )
    }
}

const RESOLUTION_SCOPE_TAG_SIZE: u32 = 2;
const RESOLUTION_SCOPE_TAG_MASK: u32 = (0x1 << RESOLUTION_SCOPE_TAG_SIZE) - 1; // 0x11

/// 2 bits tag
pub enum ResolutionScope {
    Mod = 0,
    ModRef = 1,
    // AssemblyRef = 2,
    TypeRef = 3,
}

pub fn get_typeref_parent(raw_idx: u32, tag: ResolutionScope) -> u32 {
    (raw_idx << RESOLUTION_SCOPE_TAG_SIZE) | (tag as u32)
}

impl ISerializable for TypeDef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.flag.serialize(buf);
        self.name.serialize(buf);

        self.extends.serialize(buf);

        self.fields.serialize(buf);
        self.methods.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> TypeDef {
        let flag = u32::deserialize(buf);
        let name = u32::deserialize(buf);

        let extends = u32::deserialize(buf);

        let fields = u32::deserialize(buf);
        let methods = u32::deserialize(buf);
        TypeDef {
            flag,
            name,

            extends,

            fields,
            methods,
        }
    }
}

impl ISerializable for TypeRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.parent.serialize(buf);
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let parent = u32::deserialize(buf);
        let name = u32::deserialize(buf);

        TypeRef { parent, name }
    }
}

const TYPEDEFORREF_TAG_SIZE: u32 = 2;
const TYPEDEFORREF_TAG_MASK: u32 = (0x1 << TYPEDEFORREF_TAG_SIZE) - 1; // 0x11

// 2 bit tag
pub enum TypeDefOrRef {
    TypeDef = 0,
    TypeRef = 1,
    TypeSpec = 2,
}

impl TypeDefOrRef {
    pub fn to_tok_tag(&self) -> TokTag {
        match self {
            TypeDefOrRef::TypeDef => TokTag::TypeDef,
            TypeDefOrRef::TypeRef => TokTag::TypeRef,
            TypeDefOrRef::TypeSpec => TokTag::TypeSpec,
        }
    }
}

pub struct TypeSpec {
    /// index into blob heap
    pub sig: u32,
}

impl IrFmt for TypeSpec {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        ctx.blob_heap[self.sig as usize].fmt(f, ctx)
    }
}

impl ISerializable for TypeSpec {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.sig.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        TypeSpec {
            sig: u32::deserialize(buf),
        }
    }
}
