use std::io::Read;
use std::mem::transmute;

use super::inst::Inst;
use super::ir_file::*;

impl ModuleFile {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        self.minor_version.serialize(&mut buf);
        self.major_version.serialize(&mut buf);
        self.module_name.serialize(&mut buf);
        self.constant_pool.serialize(&mut buf);
        self.classes.serialize(&mut buf);
        self.fields.serialize(&mut buf);
        self.methods.serialize(&mut buf);

        buf
    }

    pub fn from_binary(stream: Box<dyn Read>) -> ModuleFile {
        let mut buf = Deserializer::new(Box::new(stream.bytes().map(|r| r.unwrap())));

        let minor_version = u16::deserialize(&mut buf);
        let major_version = u16::deserialize(&mut buf);
        let module_name = u32::deserialize(&mut buf);
        let constant_pool = Vec::deserialize(&mut buf);
        let classes = Vec::deserialize(&mut buf);
        let fields = Vec::deserialize(&mut buf);
        let methods = Vec::deserialize(&mut buf);

        ModuleFile {
            minor_version,
            major_version,
            module_name,
            constant_pool,
            classes,
            fields,
            methods,
        }
    }
}

struct Deserializer {
    stream: Box<dyn Iterator<Item = u8>>,
    bytes_taken: u32,
}

impl Deserializer {
    fn new(stream: Box<dyn Iterator<Item = u8>>) -> Deserializer {
        Deserializer {
            stream: stream,
            bytes_taken: 0,
        }
    }

    fn take_byte(&mut self) -> u8 {
        self.bytes_taken += 1;
        (&mut self.stream).next().unwrap()
    }

    fn take_bytes2(&mut self) -> [u8; 2] {
        self.bytes_taken += 2;
        let b1 = (&mut self.stream).next().unwrap();
        let b2 = (&mut self.stream).next().unwrap();

        [b1, b2]
    }

    fn take_bytes4(&mut self) -> [u8; 4] {
        self.bytes_taken += 4;
        let b1 = (&mut self.stream).next().unwrap();
        let b2 = (&mut self.stream).next().unwrap();
        let b3 = (&mut self.stream).next().unwrap();
        let b4 = (&mut self.stream).next().unwrap();

        [b1, b2, b3, b4]
    }

    fn take_bytes(&mut self, n: u32) -> Vec<u8> {
        self.bytes_taken += n;
        (&mut self.stream).take(n as usize).collect()
    }
}

trait Serializable {
    fn serialize(&self, buf: &mut Vec<u8>);
    fn deserialize(buf: &mut Deserializer) -> Self;
}

impl Serializable for u8 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(*self)
    }

    fn deserialize(buf: &mut Deserializer) -> u8 {
        buf.take_byte()
    }
}

impl Serializable for u16 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push((self >> 8) as u8);
        buf.push(*self as u8);
    }

    fn deserialize(buf: &mut Deserializer) -> u16 {
        let v = buf.take_bytes2();
        ((v[0] as u16) << 8) + (v[1] as u16)
    }
}

impl Serializable for u32 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push((self >> 24) as u8);
        buf.push((self >> 16) as u8);
        buf.push((self >> 8) as u8);
        buf.push(*self as u8);
    }

    fn deserialize(buf: &mut Deserializer) -> u32 {
        let v = buf.take_bytes4();
        ((v[0] as u32) << 24) + ((v[1] as u32) << 16) + ((v[2] as u32) << 8) + (v[3] as u32)
    }
}

impl Serializable for i8 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        unsafe { buf.push(transmute(*self)) }
    }

    fn deserialize(buf: &mut Deserializer) -> i8 {
        unsafe { transmute(buf.take_byte()) }
    }
}

impl Serializable for i32 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let bytes = self.to_be_bytes();
        for b in bytes.iter() {
            buf.push(*b);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> i32 {
        let bytes = buf.take_bytes4();
        i32::from_be_bytes(bytes)
    }
}

impl Serializable for String {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for b in self.as_bytes() {
            b.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> String {
        let len = u16::deserialize(buf);
        let v = buf.take_bytes(len as u32);
        String::from_utf8(v).unwrap()
    }
}

impl Serializable for Vec<u8> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u32).serialize(buf); // byte vectors use a 4-byte length prefix, not 2-byte
        for b in self.into_iter() {
            b.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<u8> {
        let len = u32::deserialize(buf); // byte vectors use a 4-byte length prefix, not 2-byte
        buf.take_bytes(len)
    }
}

impl Serializable for Vec<Constant> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for constant in self.into_iter() {
            constant.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<Constant> {
        let len = u16::deserialize(buf);
        (0..len)
            .into_iter()
            .map(|_| Constant::deserialize(buf))
            .collect()
    }
}

impl Serializable for Vec<IrClass> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for cls in self.into_iter() {
            cls.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<IrClass> {
        let len = u16::deserialize(buf);
        (0..len)
            .into_iter()
            .map(|_| IrClass::deserialize(buf))
            .collect()
    }
}

impl Serializable for Vec<IrField> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for f in self.into_iter() {
            f.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<IrField> {
        let len = u16::deserialize(buf);
        (0..len)
            .into_iter()
            .map(|_| IrField::deserialize(buf))
            .collect()
    }
}

impl Serializable for Vec<IrMethod> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for m in self.into_iter() {
            m.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<IrMethod> {
        let len = u16::deserialize(buf);
        (0..len)
            .into_iter()
            .map(|_| IrMethod::deserialize(buf))
            .collect()
    }
}

impl Serializable for Vec<ExceptionTableEntry> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for e in self.into_iter() {
            e.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<ExceptionTableEntry> {
        let len = u16::deserialize(buf);
        (0..len)
            .into_iter()
            .map(|_| ExceptionTableEntry::deserialize(buf))
            .collect()
    }
}

impl Serializable for Vec<LineNumberTableEntry> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for l in self.into_iter() {
            l.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<LineNumberTableEntry> {
        let len = u16::deserialize(buf);
        (0..len)
            .into_iter()
            .map(|_| LineNumberTableEntry::deserialize(buf))
            .collect()
    }
}

impl Serializable for Vec<Inst> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let mut code = vec![];
        for inst in self.into_iter() {
            inst.serialize(&mut code);
        }
        code.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<Inst> {
        let code: Vec<u8> = Vec::deserialize(buf);
        let code_len = code.len() as u32;
        let mut code_buf = Deserializer::new(Box::new(code.into_iter()));
        let mut out = vec![];
        while code_buf.bytes_taken < code_len {
            out.push(Inst::deserialize(&mut code_buf));
        }
        out
    }
}

impl Serializable for Constant {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Constant::Utf8(string) => {
                1u8.serialize(buf);
                string.serialize(buf);
            }
            Constant::Class(name_index) => {
                7u8.serialize(buf);
                name_index.serialize(buf);
            }
            Constant::String(string_index) => {
                8u8.serialize(buf);
                string_index.serialize(buf);
            }
            Constant::Fieldref(class_index, name_and_type_index) => {
                9u8.serialize(buf);
                class_index.serialize(buf);
                name_and_type_index.serialize(buf);
            }
            Constant::Methodref(class_index, name_and_type_index) => {
                10u8.serialize(buf);
                class_index.serialize(buf);
                name_and_type_index.serialize(buf);
            }
            Constant::NameAndType(name_index, descriptor_index) => {
                12u8.serialize(buf);
                name_index.serialize(buf);
                descriptor_index.serialize(buf);
            }
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Constant {
        let code = u8::deserialize(buf);
        match code {
            1 => Constant::Utf8(String::deserialize(buf)),
            7 => Constant::Class(u32::deserialize(buf)),
            8 => Constant::String(u32::deserialize(buf)),
            9 => Constant::Fieldref(u32::deserialize(buf), u32::deserialize(buf)),
            10 => Constant::Methodref(u32::deserialize(buf), u32::deserialize(buf)),
            12 => Constant::NameAndType(u32::deserialize(buf), u32::deserialize(buf)),
            _ => panic!("Don't know how to deserialize Constant of type: {}", code),
        }
    }
}

impl Serializable for IrClass {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name_idx.serialize(buf);
        self.flag.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> IrClass {
        let name_idx = u32::deserialize(buf);
        let flag = u32::deserialize(buf);
        IrClass { name_idx, flag }
    }
}

impl Serializable for IrField {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.class_idx.serialize(buf);
        self.flag.serialize(buf);
        self.name_idx.serialize(buf);
        self.descriptor_idx.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> IrField {
        let class_idx = u16::deserialize(buf);
        let flag = u16::deserialize(buf);
        let name_idx = u32::deserialize(buf);
        let descriptor_idx = u32::deserialize(buf);

        IrField {
            class_idx,
            flag,
            name_idx,
            descriptor_idx,
        }
    }
}

impl Serializable for IrMethod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.class_idx.serialize(buf);

        self.flag.serialize(buf);
        self.name_idx.serialize(buf);
        self.descriptor_idx.serialize(buf);

        self.locals.serialize(buf);
        self.insts.serialize(buf);
        self.exception.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> IrMethod {
        let class_idx = u16::deserialize(buf);
        let flag = u16::deserialize(buf);
        let name_idx = u32::deserialize(buf);
        let descriptor_idx = u32::deserialize(buf);
        let locals = u16::deserialize(buf);
        let insts: Vec<Inst> = Vec::deserialize(buf);
        let exception: Vec<ExceptionTableEntry> = Vec::deserialize(buf);

        IrMethod {
            class_idx,
            flag,
            name_idx,
            descriptor_idx,
            locals,
            insts,
            exception,
        }
    }
}

impl Serializable for ExceptionTableEntry {
    fn serialize(&self, _buf: &mut Vec<u8>) {
        unimplemented!();
    }

    fn deserialize(_buf: &mut Deserializer) -> ExceptionTableEntry {
        unimplemented!();
    }
}

impl Serializable for LineNumberTableEntry {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.start_pc.serialize(buf);
        self.line_number.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> LineNumberTableEntry {
        LineNumberTableEntry {
            start_pc: u16::deserialize(buf),
            line_number: u16::deserialize(buf),
        }
    }
}

impl Serializable for Inst {
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
            Inst::StArgS(idx) => {
                0x10u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdLocS(idx) => {
                0x11u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdLoc(idx) => {
                0xFE0Cu16.serialize(buf);
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
            Inst::LdCS(num) => {
                0x1Fu8.serialize(buf);
                num.serialize(buf);
            }
            Inst::LdC(num) => {
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

            Inst::Add => 0x58u8.serialize(buf),

            Inst::CallVirt(idx) => {
                0x6Fu8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::New(idx) => {
                0x73u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdFld(idx) => {
                0x7Bu8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::StFld(idx) => {
                0x7Du8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::LdSFld(idx) => {
                0x7Eu8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::StSFld(idx) => {
                0x80u8.serialize(buf);
                idx.serialize(buf);
            }
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Inst {
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
            0x10 => Inst::StArgS(u8::deserialize(buf)),
            0x11 => Inst::LdLocS(u8::deserialize(buf)),
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
            0x1F => Inst::LdCS(i8::deserialize(buf)),
            0x20 => Inst::LdC(i32::deserialize(buf)),

            0x25 => Inst::Dup,
            0x26 => Inst::Pop,

            0x28 => Inst::Call(u32::deserialize(buf)),

            0x2A => Inst::Add,

            0x58 => Inst::Ret,

            0x6F => Inst::CallVirt(u32::deserialize(buf)),
            0x73 => Inst::New(u32::deserialize(buf)),
            0x7B => Inst::LdFld(u32::deserialize(buf)),
            0x7D => Inst::StFld(u32::deserialize(buf)),
            0x7E => Inst::LdSFld(u32::deserialize(buf)),
            0x80 => Inst::StSFld(u32::deserialize(buf)),

            0xFE => {
                let inner_coder = u8::deserialize(buf);
                match inner_coder {
                    0x0C => Inst::LdLoc(u16::deserialize(buf)),
                    0x0E => Inst::StLoc(u16::deserialize(buf)),
                    _ => panic!("Unknown inst 0xFE{:X}", inner_coder),
                }
            }
            _ => panic!("Unknown Inst: 0x{:X}", code),
        }
    }
}
