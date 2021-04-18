use super::bc_serde::{IDeserializer, ISerializable};

pub struct IrParam {
    /// Param attributes
    flag: u16,
    /// 0 means return type, others means normal parameters
    sequence: u16,
    /// index into str heap
    name: u32,
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
