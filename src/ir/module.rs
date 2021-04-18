use super::bc_serde::{IDeserializer, ISerializable};
use super::file::IrFile;
use super::text_serde::IrFmt;

use std::fmt;

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

impl ISerializable for IrMod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.entrypoint.serialize(buf)
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let name = u32::deserialize(buf);
        let entrypoint = u32::deserialize(buf);
        IrMod { name, entrypoint }
    }
}

impl ISerializable for IrModRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let name = u32::deserialize(buf);
        IrModRef { name }
    }
}
