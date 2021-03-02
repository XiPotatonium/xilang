use std::io::Read;

use crate::ir::bc_serde::{Deserializer, IDeserializer, ISerializable};
use crate::ir::blob::IrBlob;
use crate::ir::ir_file::*;

pub struct VMFile {
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
    pub codes: Vec<Vec<u8>>,
}

impl VMFile {
    pub fn from_binary(stream: Box<dyn Read>) -> VMFile {
        let mut buf = Deserializer::new(Box::new(stream.bytes().map(|r| r.unwrap())));

        let minor_version = u16::deserialize(&mut buf);
        let major_version = u16::deserialize(&mut buf);

        if major_version != MAJOR_VERSION || minor_version != MINOR_VERSION {
            println!(
                "Warning: Incompatible file version {}.{}  VM version: {}.{}",
                major_version, minor_version, MAJOR_VERSION, MINOR_VERSION
            );
        }

        let mod_tbl = Vec::deserialize(&mut buf);
        let modref_tbl = Vec::deserialize(&mut buf);

        let type_tbl = Vec::deserialize(&mut buf);
        let typeref_tbl = Vec::deserialize(&mut buf);

        let field_tbl = Vec::deserialize(&mut buf);
        let method_tbl = Vec::deserialize(&mut buf);
        let memberref_tbl = Vec::deserialize(&mut buf);

        let str_heap = Vec::deserialize(&mut buf);
        let blob_heap = Vec::deserialize(&mut buf);

        let codes = Vec::deserialize(&mut buf);

        VMFile {
            minor_version,
            major_version,

            mod_tbl,
            modref_tbl,

            class_tbl: type_tbl,
            classref_tbl: typeref_tbl,

            field_tbl,
            method_tbl,
            memberref_tbl,

            str_heap,
            blob_heap,

            codes,
        }
    }

    pub fn mod_name(&self) -> &str {
        &self.str_heap[self.mod_tbl[0].name as usize]
    }
}

impl ISerializable for Vec<Vec<u8>> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let mut code = vec![];
        for inst in self.iter() {
            inst.serialize(&mut code);
        }
        code.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let code: Vec<u8> = Vec::deserialize(buf);
        let code_len = code.len() as u32;
        let mut code_buf = Deserializer::new(Box::new(code.into_iter()));
        let mut out = vec![];
        while code_buf.bytes_taken < code_len {
            out.push(Vec::deserialize(&mut code_buf));
        }
        out
    }
}