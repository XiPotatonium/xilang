use super::attrib::{ParamAttrib, ParamAttribFlag};
use super::bc_serde::{IDeserializer, ISerializable};
use super::file::IrFile;
use super::text_serde::IrFmt;

use std::fmt;

pub struct IrParam {
    /// Param attributes
    pub flag: u16,
    /// 0 means return type, others means normal parameters
    pub sequence: u16,
    /// index into str heap
    pub name: u32,
}

impl ISerializable for IrParam {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.flag.serialize(buf);
        self.sequence.serialize(buf);
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let flag = u16::deserialize(buf);
        let sequence = u16::deserialize(buf);
        let name = u32::deserialize(buf);

        IrParam {
            flag,
            sequence,
            name,
        }
    }
}

impl IrFmt for IrParam {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        let name = ctx.get_str(self.name);
        let flag = ParamAttrib::from(self.flag);

        if name.len() != 0 {
            write!(f, "{}: ", name)?;
        }

        if !flag.is(ParamAttribFlag::Default) {
            // default flag will not display
            write!(f, "{}", flag)?;
        }

        Ok(())
    }
}
