use std::io::Read;
use std::mem::transmute;

use super::super::inst::Inst;
use super::class_file::*;

impl ClassFile {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        self.magic.serialize(&mut buf);
        self.minor_version.serialize(&mut buf);
        self.major_version.serialize(&mut buf);
        self.constant_pool.serialize(&mut buf);
        self.access_flags.serialize(&mut buf);
        self.this_class.serialize(&mut buf);
        self.interfaces.serialize(&mut buf);
        self.fields.serialize(&mut buf);
        self.methods.serialize(&mut buf);

        buf
    }

    pub fn from_binary(stream: Box<dyn Read>) -> ClassFile {
        let mut buf = Deserializer::new(Box::new(stream.bytes().map(|r| r.unwrap())));
        let magic = u32::deserialize(&mut buf);
        let minor_version = u16::deserialize(&mut buf);
        let major_version = u16::deserialize(&mut buf);
        let constant_pool = Vec::deserialize(&mut buf);
        let access_flags = u16::deserialize(&mut buf);
        let this_class = u16::deserialize(&mut buf);
        let interfaces = Vec::deserialize(&mut buf);
        let fields = Vec::deserialize(&mut buf);
        let methods = Vec::deserialize(&mut buf);

        ClassFile {
            magic,
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            interfaces,
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

impl Serializable for Vec<IrInterface> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for constant in self.into_iter() {
            constant.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<IrInterface> {
        let len = u16::deserialize(buf);
        (0..len)
            .into_iter()
            .map(|_| IrInterface::deserialize(buf))
            .collect()
    }
}

impl Serializable for Vec<IrField> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for constant in self.into_iter() {
            constant.serialize(buf);
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
        for constant in self.into_iter() {
            constant.serialize(buf);
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
        for constant in self.into_iter() {
            constant.serialize(buf);
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
        for constant in self.into_iter() {
            constant.serialize(buf);
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
        let mut code_buf = &mut Deserializer::new(Box::new(code.into_iter()));
        let mut out = vec![];
        while code_buf.bytes_taken < code_len {
            out.push(Inst::deserialize(code_buf));
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
            Constant::Integer(i) => {
                3u8.serialize(buf);
                i.serialize(buf);
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
            7 => Constant::Class(u16::deserialize(buf)),
            8 => Constant::String(u16::deserialize(buf)),
            9 => Constant::Fieldref(u16::deserialize(buf), u16::deserialize(buf)),
            10 => Constant::Methodref(u16::deserialize(buf), u16::deserialize(buf)),
            12 => Constant::NameAndType(u16::deserialize(buf), u16::deserialize(buf)),
            _ => panic!("Don't know how to deserialize Constant of type: {}", code),
        }
    }
}

impl Serializable for IrInterface {
    fn serialize(&self, _buf: &mut Vec<u8>) {
        unimplemented!("TODO implement IrInterface::serialize")
    }

    fn deserialize(buf: &mut Deserializer) -> IrInterface {
        unimplemented!("TODO implement IrInterface::deserialize")
    }
}

impl Serializable for IrField {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.access_flags.serialize(buf);
        self.name_index.serialize(buf);
        self.descriptor_index.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> IrField {
        let access_flags = u16::deserialize(buf);
        let name_index = u16::deserialize(buf);
        let descriptor_index = u16::deserialize(buf);

        IrField {
            access_flags,
            name_index,
            descriptor_index,
        }
    }
}

impl Serializable for IrMethod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.access_flags.serialize(buf);
        self.name_index.serialize(buf);
        self.descriptor_index.serialize(buf);

        self.locals_stack.serialize(buf);
        self.insts.serialize(buf);
        self.exception.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> IrMethod {
        let access_flags = u16::deserialize(buf);
        let name_index = u16::deserialize(buf);
        let descriptor_index = u16::deserialize(buf);
        let locals_stack = u16::deserialize(buf);
        let insts: Vec<Inst> = Vec::deserialize(buf);
        let exception: Vec<ExceptionTableEntry> = Vec::deserialize(buf);

        IrMethod {
            access_flags,
            name_index,
            descriptor_index,
            locals_stack,
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
            Inst::IConstM1 => 0x02u8.serialize(buf),
            Inst::IConst0 => 0x03u8.serialize(buf),
            Inst::IConst1 => 0x04u8.serialize(buf),
            Inst::IConst2 => 0x05u8.serialize(buf),
            Inst::IConst3 => 0x6u8.serialize(buf),
            Inst::IConst4 => 0x07u8.serialize(buf),
            Inst::IConst5 => 0x08u8.serialize(buf),
            Inst::BIPush(val) => {
                0x10u8.serialize(buf);
                val.serialize(buf);
            }
            Inst::LdC(idx) => {
                0x12u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::ILoad(idx) => {
                0x15u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::ALoad(idx) => {
                0x19u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::ILoad0 => 0x1Au8.serialize(buf),
            Inst::ILoad1 => 0x1Bu8.serialize(buf),
            Inst::ILoad2 => 0x1Cu8.serialize(buf),
            Inst::ILoad3 => 0x1Du8.serialize(buf),
            Inst::ALoad0 => 0x2Au8.serialize(buf),
            Inst::ALoad1 => 0x2Bu8.serialize(buf),
            Inst::ALoad2 => 0x2Cu8.serialize(buf),
            Inst::ALoad3 => 0x2Du8.serialize(buf),
            Inst::IStore(idx) => {
                0x36u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::AStore(idx) => {
                0x3Au8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::IStore0 => 0x3Bu8.serialize(buf),
            Inst::IStore1 => 0x3Cu8.serialize(buf),
            Inst::IStore2 => 0x3Du8.serialize(buf),
            Inst::IStore3 => 0x3Eu8.serialize(buf),
            Inst::AStore0 => 0x4Bu8.serialize(buf),
            Inst::AStore1 => 0x4Cu8.serialize(buf),
            Inst::AStore2 => 0x4Eu8.serialize(buf),
            Inst::AStore3 => 0x4Eu8.serialize(buf),
            Inst::Pop => 0x57u8.serialize(buf),
            Inst::Pop2 => 0x58u8.serialize(buf),
            Inst::IAdd => 0x60u8.serialize(buf),
            Inst::Return => 0xB1u8.serialize(buf),
            Inst::GetStatic(idx) => {
                0xB2u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::PutStatic(idx) => {
                0xB3u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::GetField(idx) => {
                0xB4u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::PutField(idx) => {
                0xB5u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::InvokeVirtual(idx) => {
                0xB6u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::InvokeSpecial(idx) => {
                0xB7u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::InvokeStatic(idx) => {
                0xB8u8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::New(idx) => {
                0xBBu8.serialize(buf);
                idx.serialize(buf);
            }
            Inst::ArrayLength => 0xBEu8.serialize(buf),
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Inst {
        let code = u8::deserialize(buf);
        match code {
            0x02 => Inst::IConstM1,
            0x03 => Inst::IConst0,
            0x04 => Inst::IConst1,
            0x05 => Inst::IConst2,
            0x06 => Inst::IConst3,
            0x07 => Inst::IConst4,
            0x08 => Inst::IConst5,
            0x10 => Inst::BIPush(i8::deserialize(buf)),
            0x12 => Inst::LdC(u16::deserialize(buf)),
            0x15 => Inst::ILoad(u16::deserialize(buf)),
            0x19 => Inst::ALoad(u16::deserialize(buf)),
            0x1A => Inst::ILoad0,
            0x1B => Inst::ILoad1,
            0x1C => Inst::ILoad2,
            0x1D => Inst::ILoad3,
            0x2A => Inst::ALoad0,
            0x2B => Inst::ALoad1,
            0x2C => Inst::ALoad2,
            0x2D => Inst::ALoad3,
            0x36 => Inst::IStore(u16::deserialize(buf)),
            0x3A => Inst::AStore(u16::deserialize(buf)),
            0x3B => Inst::IStore0,
            0x3C => Inst::IStore1,
            0x3D => Inst::IStore2,
            0x3E => Inst::IStore3,
            0x4B => Inst::AStore0,
            0x4C => Inst::AStore1,
            0x4D => Inst::AStore2,
            0x4E => Inst::AStore3,
            0x57 => Inst::Pop,
            0x58 => Inst::Pop2,
            0x60 => Inst::IAdd,
            0xB1 => Inst::Return,
            0xB2 => Inst::GetStatic(u16::deserialize(buf)),
            0xB3 => Inst::PutStatic(u16::deserialize(buf)),
            0xB4 => Inst::GetField(u16::deserialize(buf)),
            0xB5 => Inst::GetStatic(u16::deserialize(buf)),
            0xB6 => Inst::InvokeVirtual(u16::deserialize(buf)),
            0xB7 => Inst::InvokeSpecial(u16::deserialize(buf)),
            0xB8 => Inst::InvokeStatic(u16::deserialize(buf)),
            0xBB => Inst::New(u16::deserialize(buf)),
            0xBE => Inst::ArrayLength,
            _ => panic!("Unknown Inst: 0x{:X}", code),
        }
    }
}
