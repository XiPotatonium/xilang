use super::super::bc_serde::{IDeserializer, ISerializable};
use super::Inst;

/// III.1.2.1 Opcode encodings
impl ISerializable for Inst {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Inst::Nop => 0x00u8.serialize(buf),

            Inst::LdArg0 => 0x02u8.serialize(buf),
            Inst::LdArg1 => 0x03u8.serialize(buf),
            Inst::LdArg2 => 0x04u8.serialize(buf),
            Inst::LdArg3 => 0x05u8.serialize(buf),
            Inst::LdLoc0 => 0x06u8.serialize(buf),
            Inst::LdLoc1 => 0x07u8.serialize(buf),
            Inst::LdLoc2 => 0x08u8.serialize(buf),
            Inst::LdLoc3 => 0x09u8.serialize(buf),
            Inst::StLoc0 => 0x0Au8.serialize(buf),
            Inst::StLoc1 => 0x0Bu8.serialize(buf),
            Inst::StLoc2 => 0x0Cu8.serialize(buf),
            Inst::StLoc3 => 0x0Du8.serialize(buf),

            Inst::LdArgS(idx) => {
                0x0Eu8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdArgAS(idx) => {
                0x0Fu8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::StArgS(idx) => {
                0x10u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdLocS(idx) => {
                0x11u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdLocAS(idx) => {
                0x12u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdLoc(idx) => {
                0xFE0Cu16.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdLocA(idx) => {
                0xFE0Du16.serialize(buf);
                idx.serialize(buf);
            }
            Inst::StLocS(idx) => {
                0x13u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::StLoc(idx) => {
                0xFE0Eu16.serialize(buf);
                idx.serialize(buf);
            }

            Inst::LdNull => 0x14u8.serialize(buf),
            Inst::LdCM1 => 0x15u8.serialize(buf),
            Inst::LdC0 => 0x16u8.serialize(buf),
            Inst::LdC1 => 0x17u8.serialize(buf),
            Inst::LdC2 => 0x18u8.serialize(buf),
            Inst::LdC3 => 0x19u8.serialize(buf),
            Inst::LdC4 => 0x1Au8.serialize(buf),
            Inst::LdC5 => 0x1Bu8.serialize(buf),
            Inst::LdC6 => 0x1Cu8.serialize(buf),
            Inst::LdC7 => 0x1Du8.serialize(buf),
            Inst::LdC8 => 0x1Eu8.serialize(buf),
            Inst::LdCI4S(num) => {
                0x1Fu8.serialize(buf);
                num.serialize(buf);
            }
            Inst::LdCI4(num) => {
                0x20u8.serialize(buf);
                num.serialize(buf);
            }

            Inst::Dup => 0x25u8.serialize(buf),
            Inst::Pop => 0x26u8.serialize(buf),

            Inst::Call(idx) => {
                0x28u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::Ret => 0x2Au8.serialize(buf),

            Inst::Br(offset) => {
                0x38u8.serialize(buf);
                offset.serialize(buf);
            }
            Inst::BrFalse(offset) => {
                0x39u8.serialize(buf);
                offset.serialize(buf);
            }
            Inst::BrTrue(offset) => {
                0x3Au8.serialize(buf);
                offset.serialize(buf);
            }
            Inst::BEq(offset) => {
                0x3Bu8.serialize(buf);
                offset.serialize(buf);
            }
            Inst::BGe(offset) => {
                0x3Cu8.serialize(buf);
                offset.serialize(buf);
            }
            Inst::BGt(offset) => {
                0x3Du8.serialize(buf);
                offset.serialize(buf);
            }
            Inst::BLe(offset) => {
                0x3Eu8.serialize(buf);
                offset.serialize(buf);
            }
            Inst::BLt(offset) => {
                0x3Fu8.serialize(buf);
                offset.serialize(buf);
            }

            Inst::CEq => 0xFE01u16.serialize(buf),
            Inst::CGt => 0xFE02u16.serialize(buf),
            Inst::CLt => 0xFE04u16.serialize(buf),

            Inst::Add => 0x58u8.serialize(buf),
            Inst::Sub => 0x59u8.serialize(buf),
            Inst::Mul => 0x5Au8.serialize(buf),
            Inst::Div => 0x5Bu8.serialize(buf),
            Inst::Rem => 0x5Du8.serialize(buf),

            Inst::Neg => 0x65u8.serialize(buf),

            Inst::CallVirt(idx) => {
                0x6Fu8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdStr(tok) => {
                0x72u8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::NewObj(tok) => {
                0x73u8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::LdFld(tok) => {
                0x7Bu8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::LdFldA(tok) => {
                0x7Cu8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::StFld(tok) => {
                0x7Du8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::LdSFld(tok) => {
                0x7Eu8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::LdSFldA(tok) => {
                0x7Fu8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::StSFld(tok) => {
                0x80u8.serialize(buf);
                tok.serialize(buf);
            }

            Inst::NewArr(tok) => {
                0x8Du8.serialize(buf);
                tok.serialize(buf);
            }

            Inst::LdLen => 0x8Eu8.serialize(buf),

            Inst::LdElemI4 => 0x94u8.serialize(buf),
            Inst::LdElemRef => 0x9Au8.serialize(buf),
            Inst::StElemI4 => 0x9Eu8.serialize(buf),
            Inst::StElemRef => 0xA2u8.serialize(buf),

            Inst::LdElem(tok) => {
                0xA3u8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::LdElemA(tok) => {
                0x8Fu8.serialize(buf);
                tok.serialize(buf);
            }
            Inst::StElem(tok) => {
                0xA4u8.serialize(buf);
                tok.serialize(buf);
            }

            Inst::InitObj(tok) => {
                0xFE15u16.serialize(buf);
                tok.serialize(buf);
            }
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Inst {
        let code = u8::deserialize(buf);
        match code {
            0x00 => Inst::Nop,

            0x02 => Inst::LdArg0,
            0x03 => Inst::LdArg1,
            0x04 => Inst::LdArg2,
            0x05 => Inst::LdArg3,

            0x06 => Inst::LdLoc0,
            0x07 => Inst::LdLoc1,
            0x08 => Inst::LdLoc2,
            0x09 => Inst::LdLoc3,
            0x0A => Inst::StLoc0,
            0x0B => Inst::StLoc1,
            0x0C => Inst::StLoc2,
            0x0D => Inst::StLoc3,

            0x0E => Inst::LdArgS(u8::deserialize(buf)),
            0x0F => Inst::LdArgAS(u8::deserialize(buf)),
            0x10 => Inst::StArgS(u8::deserialize(buf)),
            0x11 => Inst::LdLocS(u8::deserialize(buf)),
            0x12 => Inst::LdLocAS(u8::deserialize(buf)),
            0x13 => Inst::StLocS(u8::deserialize(buf)),

            0x14 => Inst::LdNull,
            0x15 => Inst::LdCM1,
            0x16 => Inst::LdC0,
            0x17 => Inst::LdC1,
            0x18 => Inst::LdC2,
            0x19 => Inst::LdC3,
            0x1A => Inst::LdC4,
            0x1B => Inst::LdC5,
            0x1C => Inst::LdC6,
            0x1D => Inst::LdC7,
            0x1E => Inst::LdC8,
            0x1F => Inst::LdCI4S(i8::deserialize(buf)),
            0x20 => Inst::LdCI4(i32::deserialize(buf)),

            0x25 => Inst::Dup,
            0x26 => Inst::Pop,

            0x28 => Inst::Call(u32::deserialize(buf)),

            0x2A => Inst::Ret,

            0x38 => Inst::Br(i32::deserialize(buf)),
            0x39 => Inst::BrFalse(i32::deserialize(buf)),
            0x3A => Inst::BrTrue(i32::deserialize(buf)),
            0x3B => Inst::BEq(i32::deserialize(buf)),
            0x3C => Inst::BGe(i32::deserialize(buf)),
            0x3D => Inst::BGt(i32::deserialize(buf)),
            0x3E => Inst::BLe(i32::deserialize(buf)),
            0x3F => Inst::BLt(i32::deserialize(buf)),

            0x58 => Inst::Add,
            0x59 => Inst::Sub,
            0x5A => Inst::Mul,
            0x5B => Inst::Div,
            0x5D => Inst::Rem,

            0x65 => Inst::Neg,

            0x6F => Inst::CallVirt(u32::deserialize(buf)),
            0x72 => Inst::LdStr(u32::deserialize(buf)),
            0x73 => Inst::NewObj(u32::deserialize(buf)),
            0x7B => Inst::LdFld(u32::deserialize(buf)),
            0x7C => Inst::LdFldA(u32::deserialize(buf)),
            0x7D => Inst::StFld(u32::deserialize(buf)),
            0x7E => Inst::LdSFld(u32::deserialize(buf)),
            0x7F => Inst::LdSFldA(u32::deserialize(buf)),
            0x80 => Inst::StSFld(u32::deserialize(buf)),

            0x8D => Inst::NewArr(u32::deserialize(buf)),
            0x8E => Inst::LdLen,
            0x8F => Inst::LdElemA(u32::deserialize(buf)),
            0x94 => Inst::LdElemI4,
            0x9A => Inst::LdElemRef,
            0x9E => Inst::StElemI4,
            0xA2 => Inst::StElemRef,
            0xA3 => Inst::LdElem(u32::deserialize(buf)),
            0xA4 => Inst::StElem(u32::deserialize(buf)),

            0xFE => {
                let inner_code = u8::deserialize(buf);
                match inner_code {
                    0x01 => Inst::CEq,
                    0x02 => Inst::CGt,
                    0x04 => Inst::CLt,
                    0x0C => Inst::LdLoc(u16::deserialize(buf)),
                    0x0D => Inst::LdLocA(u16::deserialize(buf)),
                    0x0E => Inst::StLoc(u16::deserialize(buf)),
                    0x15 => Inst::InitObj(u32::deserialize(buf)),
                    _ => panic!("Unknown inst 0xFE{:X}", inner_code),
                }
            }
            _ => panic!("Unknown inst: 0x{:X}", code),
        }
    }
}
