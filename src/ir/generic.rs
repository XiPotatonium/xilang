use super::bc_serde::{IDeserializer, ISerializable};

/// II.22.20 0x2A
pub struct GenericParam {
    /// index, from 0, from left to right
    pub number: u16,
    /// GenericParamAttrib
    pub flag: u16,
    /// TypeOrMethodDef
    pub owner: u32,
    /// non-null index into str heap, used only for compiler and reflection
    pub name: u32,
}

impl ISerializable for GenericParam {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.number.serialize(buf);
        self.flag.serialize(buf);
        self.owner.serialize(buf);
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let number = u16::deserialize(buf);
        let flag = u16::deserialize(buf);
        let owner = u32::deserialize(buf);
        let name = u32::deserialize(buf);
        Self {
            number,
            flag,
            owner,
            name,
        }
    }
}

/// II.22.21 0x2C
pub struct GenericParamConstraint {
    /// index into GenericParam table
    pub owner: u32,
    /// TypeDefOrRef
    pub constraint: u32,
}

impl ISerializable for GenericParamConstraint {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.owner.serialize(buf);
        self.constraint.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let owner = u32::deserialize(buf);
        let constraint = u32::deserialize(buf);
        Self { owner, constraint }
    }
}
