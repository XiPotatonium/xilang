use super::bc_serde::{IDeserializer, ISerializable};
use super::file::IrFile;
use super::text_serde::IrFmt;

use std::fmt;

pub struct Mod {
    /// index into str heap
    pub name: u32,

    /// index of codes
    pub entrypoint: u32,
}

impl IrFmt for Mod {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        write!(f, "{}", ctx.get_str(self.name))
    }
}

pub struct ModRef {
    /// index into str heap
    pub name: u32,
}

impl IrFmt for ModRef {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        write!(f, "{}", ctx.get_str(self.name))
    }
}

impl ISerializable for Mod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.entrypoint.serialize(buf)
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let name = u32::deserialize(buf);
        let entrypoint = u32::deserialize(buf);
        Mod { name, entrypoint }
    }
}

impl ISerializable for ModRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let name = u32::deserialize(buf);
        ModRef { name }
    }
}
