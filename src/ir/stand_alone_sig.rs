use super::bc_serde::{IDeserializer, ISerializable};

pub struct IrStandAloneSig {
    /// index into blob heap
    pub sig: u32,
}

impl ISerializable for IrStandAloneSig {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.sig.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        IrStandAloneSig {
            sig: u32::deserialize(buf),
        }
    }
}
