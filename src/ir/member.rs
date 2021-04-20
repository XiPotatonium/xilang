use super::bc_serde::{IDeserializer, ISerializable};
use super::file::IrFile;
use super::text_serde::IrFmt;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct MemberRef {
    /// index into MemberRefParent tbl
    pub parent: u32,
    /// index into str heap
    pub name: u32,
    /// index into blob heap
    pub sig: u32,
}

impl IrFmt for MemberRef {
    fn fmt(&self, f: &mut std::fmt::Formatter, ctx: &IrFile) -> std::fmt::Result {
        let (tag, idx) = self.get_parent();

        match tag {
            MemberRefParent::TypeDef => ctx.typedef_tbl[idx as usize - 1].fmt(f, ctx)?,
            MemberRefParent::TypeRef => ctx.typeref_tbl[idx as usize - 1].fmt(f, ctx)?,
            MemberRefParent::ModRef => ctx.modref_tbl[idx as usize - 1].fmt(f, ctx)?,
            MemberRefParent::MethodDef => ctx.method_tbl[idx as usize - 1].fmt(f, ctx)?,
        };
        write!(f, "::{}: ", ctx.get_str(self.name))?;
        ctx.blob_heap[self.sig as usize].fmt(f, ctx)
    }
}

impl MemberRef {
    pub fn get_parent(&self) -> (MemberRefParent, u32) {
        let tag = self.parent & MEMBER_REF_PARENT_TAG_MASK;
        let index = self.parent >> MEMBER_REF_PARENT_TAG_SIZE;
        if index == 0 {
            panic!("Memberred has no parent");
        }

        (
            match tag {
                0 => MemberRefParent::TypeDef,
                1 => MemberRefParent::TypeRef,
                2 => MemberRefParent::ModRef,
                3 => MemberRefParent::MethodDef,
                4 => unreachable!(),
                _ => unreachable!(),
            },
            index,
        )
    }
}

const MEMBER_REF_PARENT_TAG_SIZE: u32 = 3;
const MEMBER_REF_PARENT_TAG_MASK: u32 = (0x1 << MEMBER_REF_PARENT_TAG_SIZE) - 1; // 0x111

/// 3 bits tag
pub enum MemberRefParent {
    TypeDef = 0,
    TypeRef = 1,
    ModRef = 2,
    MethodDef = 3,
    // TypeSpec = 4
}

pub fn to_memberref_parent(raw_idx: u32, tag: MemberRefParent) -> u32 {
    (raw_idx << MEMBER_REF_PARENT_TAG_SIZE) | (tag as u32)
}

pub struct Field {
    /// index into str heap
    pub name: u32,
    /// index into blob heap
    pub sig: u32,

    /// IrFieldAttrib
    pub flag: u16,
}

impl IrFmt for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter, ctx: &IrFile) -> std::fmt::Result {
        write!(f, "{}: ", ctx.get_str(self.name))?;
        ctx.blob_heap[self.sig as usize].fmt(f, ctx)
    }
}

pub struct MethodDef {
    /// index into code tbl, similar to RVA
    pub body: u32,

    /// IrMethodImplAttrib
    pub impl_flag: u16,
    /// IrMethodAttrib
    pub flag: u16,

    /// index into str heap
    pub name: u32,
    /// index into blob heap
    pub sig: u32,
    /// index into param table
    pub param_list: u32,
}

impl IrFmt for MethodDef {
    fn fmt(&self, f: &mut std::fmt::Formatter, ctx: &IrFile) -> std::fmt::Result {
        write!(f, "{}: ", ctx.get_str(self.name))?;
        ctx.blob_heap[self.sig as usize].fmt(f, ctx)
    }
}

pub struct ImplMap {
    /// index into MemberForwarded tbl
    pub member: u32,
    /// index into str heap
    pub name: u32,
    /// index into modref tbl
    pub scope: u32,
    /// IrPInvokeAttrib
    pub flag: u16,
}

impl ImplMap {
    pub fn get_member(&self) -> (MemberForwarded, u32) {
        let tag = self.member & MEMBER_FORWARDED_TAG_MASK;
        let idx = self.member >> MEMBER_FORWARDED_TAG_SIZE;
        (
            match tag {
                0 => MemberForwarded::Field,
                1 => MemberForwarded::MethodDef,
                _ => unreachable!(),
            },
            idx,
        )
    }
}

const MEMBER_FORWARDED_TAG_SIZE: u32 = 1;
const MEMBER_FORWARDED_TAG_MASK: u32 = (0x1 << MEMBER_FORWARDED_TAG_SIZE) - 1; // 0x1

/// 1 bits tag
#[derive(Debug, PartialEq, Eq)]
pub enum MemberForwarded {
    Field = 0, // actually unreachable
    MethodDef = 1,
}

pub fn to_implmap_member(raw_idx: u32, tag: MemberForwarded) -> u32 {
    (raw_idx << MEMBER_FORWARDED_TAG_SIZE) | (tag as u32)
}

impl ISerializable for Field {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.flag.serialize(buf);
        self.name.serialize(buf);
        self.sig.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Field {
        let flag = u16::deserialize(buf);
        let name = u32::deserialize(buf);
        let descriptor = u32::deserialize(buf);

        Field {
            flag,
            name,
            sig: descriptor,
        }
    }
}

impl ISerializable for MethodDef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.body.serialize(buf);

        self.impl_flag.serialize(buf);
        self.flag.serialize(buf);

        self.name.serialize(buf);
        self.sig.serialize(buf);
        self.param_list.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> MethodDef {
        let body = u32::deserialize(buf);

        let impl_flag = u16::deserialize(buf);
        let flag = u16::deserialize(buf);

        let name = u32::deserialize(buf);
        let sig = u32::deserialize(buf);

        let param_list = u32::deserialize(buf);

        MethodDef {
            name,
            body,
            sig,
            flag,
            impl_flag,
            param_list,
        }
    }
}

impl ISerializable for MemberRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.parent.serialize(buf);
        self.name.serialize(buf);
        self.sig.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let parent = u32::deserialize(buf);
        let name = u32::deserialize(buf);
        let signature = u32::deserialize(buf);
        MemberRef {
            parent,
            name,
            sig: signature,
        }
    }
}

impl ISerializable for ImplMap {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.member.serialize(buf);
        self.name.serialize(buf);
        self.scope.serialize(buf);
        self.flag.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let member = u32::deserialize(buf);
        let name = u32::deserialize(buf);
        let scope = u32::deserialize(buf);
        let flag = u16::deserialize(buf);
        ImplMap {
            member,
            name,
            scope,
            flag,
        }
    }
}
