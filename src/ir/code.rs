use super::bc_serde::{IDeserializer, ISerializable};
use super::inst::Inst;

use std::iter::Peekable;
use std::slice::Iter;

/// Similar to fat format
pub struct CorILMethod {
    /// max stack
    pub max_stack: u16,
    /// local sig, index into StandAloneSig table, or 0 if no local var is presented
    pub locals: u32,
    pub insts: Vec<u8>,
}

impl ISerializable for CorILMethod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.max_stack.serialize(buf);
        self.locals.serialize(buf);
        self.insts.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let max_stack = u16::deserialize(buf);
        let local = u32::deserialize(buf);
        let insts = Vec::deserialize(buf);
        CorILMethod {
            max_stack,
            locals: local,
            insts,
        }
    }
}

struct InstDeserializer<'i> {
    stream: Peekable<Iter<'i, u8>>,
    bytes_taken: u32,
}

impl<'i> InstDeserializer<'i> {
    fn new(insts: &Vec<u8>) -> InstDeserializer {
        InstDeserializer {
            stream: insts.iter().peekable(),
            bytes_taken: 0,
        }
    }
}

impl<'i> IDeserializer for InstDeserializer<'i> {
    fn peek_byte(&mut self) -> u8 {
        **(self.stream.peek().unwrap())
    }

    fn take_byte(&mut self) -> u8 {
        self.bytes_taken += 1;
        *(&mut self.stream).next().unwrap()
    }

    fn take_bytes2(&mut self) -> [u8; 2] {
        self.bytes_taken += 2;
        let b1 = *(&mut self.stream).next().unwrap();
        let b2 = *(&mut self.stream).next().unwrap();

        [b1, b2]
    }

    fn take_bytes4(&mut self) -> [u8; 4] {
        self.bytes_taken += 4;
        let b1 = *(&mut self.stream).next().unwrap();
        let b2 = *(&mut self.stream).next().unwrap();
        let b3 = *(&mut self.stream).next().unwrap();
        let b4 = *(&mut self.stream).next().unwrap();

        [b1, b2, b3, b4]
    }

    fn take_bytes(&mut self, n: u32) -> Vec<u8> {
        self.bytes_taken += n;
        (&mut self.stream).take(n as usize).map(|b| *b).collect()
    }
}

impl CorILMethod {
    pub fn new(max_stack: u16, locals_sig: u32, insts: Vec<Inst>) -> CorILMethod {
        let mut code = vec![];
        for inst in insts.iter() {
            inst.serialize(&mut code);
        }
        CorILMethod {
            max_stack,
            locals: locals_sig,
            insts: code,
        }
    }

    pub fn to_insts(&self) -> Vec<Inst> {
        let mut inst_deser = InstDeserializer::new(&self.insts);
        let mut out = vec![];
        while inst_deser.bytes_taken < self.insts.len() as u32 {
            out.push(Inst::deserialize(&mut inst_deser));
        }
        out
    }
}
