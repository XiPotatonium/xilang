use super::bc_serde::{IDeserializer, ISerializable};
use super::file::IrFile;
use super::text_serde::IrFmt;

pub struct TypeDef {
    /// index into str heap
    pub name: u32,
    /// IrTypeFlag
    pub flag: u32,

    /// index into field tbl
    pub fields: u32,
    /// into into method tbl
    pub methods: u32,
}

impl IrFmt for TypeDef {
    fn fmt(&self, f: &mut std::fmt::Formatter, ctx: &IrFile) -> std::fmt::Result {
        write!(f, "{}/{}", ctx.mod_name(), ctx.get_str(self.name))
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
    pub fn get_parent(&self) -> (ResolutionScope, u32) {
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
            raw_idx,
        )
    }
}

impl IrFmt for TypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter, ctx: &IrFile) -> std::fmt::Result {
        let (parent_tag, parent_idx) = self.get_parent();
        write!(
            f,
            "{}/{}",
            match parent_tag {
                ResolutionScope::Mod => unimplemented!(),
                ResolutionScope::ModRef =>
                    ctx.get_str(ctx.modref_tbl[parent_idx as usize - 1].name),
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
    Mod = 0, // Impossible in xilang implementation
    ModRef = 1,
    // AssemblyRef = 2,
    TypeRef = 3,
}

pub fn get_typeref_parent(raw_idx: u32, tag: ResolutionScope) -> u32 {
    (raw_idx << RESOLUTION_SCOPE_TAG_SIZE) | (tag as u32)
}

impl ISerializable for TypeDef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.flag.serialize(buf);

        self.fields.serialize(buf);
        self.methods.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> TypeDef {
        let name = u32::deserialize(buf);
        let flag = u32::deserialize(buf);
        let fields = u32::deserialize(buf);
        let methods = u32::deserialize(buf);
        TypeDef {
            name,
            flag,
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
