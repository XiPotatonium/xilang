use std::io::Read;
use std::mem::transmute;

use super::inst::Inst;
use super::ir_file::*;

impl IrFile {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        self.minor_version.serialize(&mut buf);
        self.major_version.serialize(&mut buf);

        self.mod_tbl.serialize(&mut buf);
        self.modref_tbl.serialize(&mut buf);

        self.class_tbl.serialize(&mut buf);
        self.classref_tbl.serialize(&mut buf);

        self.field_tbl.serialize(&mut buf);
        self.method_tbl.serialize(&mut buf);

        self.memberref_tbl.serialize(&mut buf);

        self.str_heap.serialize(&mut buf);
        self.blob_heap.serialize(&mut buf);

        self.codes.serialize(&mut buf);

        buf
    }

    pub fn from_binary(stream: Box<dyn Read>) -> IrFile {
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

        IrFile {
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
        (self.len() as u32).serialize(buf);
        for b in self.iter() {
            b.serialize(buf);
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Vec<u8> {
        let len = u32::deserialize(buf);
        buf.take_bytes(len)
    }
}

macro_rules! impl_vec_serde {
    ($t: ident) => {
        impl Serializable for Vec<$t> {
            fn serialize(&self, buf: &mut Vec<u8>) {
                (self.len() as u32).serialize(buf);
                for v in self.iter() {
                    v.serialize(buf);
                }
            }

            fn deserialize(buf: &mut Deserializer) -> Self {
                let len = u32::deserialize(buf);
                (0..len).into_iter().map(|_| $t::deserialize(buf)).collect()
            }
        }
    };
}

impl_vec_serde!(String);
impl_vec_serde!(u32);
impl_vec_serde!(IrMod);
impl_vec_serde!(IrModRef);
impl_vec_serde!(IrClass);
impl_vec_serde!(IrClassRef);
impl_vec_serde!(IrField);
impl_vec_serde!(IrMethod);
impl_vec_serde!(IrMemberRef);
impl_vec_serde!(IrBlob);

impl Serializable for Vec<Inst> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let mut code = vec![];
        for inst in self.iter() {
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

impl Serializable for Vec<Vec<Inst>> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let mut code = vec![];
        for inst in self.iter() {
            inst.serialize(&mut code);
        }
        code.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> Self {
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

impl Serializable for IrClass {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.flag.serialize(buf);

        self.fields.serialize(buf);
        self.methods.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> IrClass {
        let name = u32::deserialize(buf);
        let flag = u32::deserialize(buf);
        let fields = u32::deserialize(buf);
        let methods = u32::deserialize(buf);
        IrClass {
            name,
            flag,
            fields,
            methods,
        }
    }
}

impl Serializable for IrClassRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.parent.serialize(buf);
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> Self {
        let parent = u32::deserialize(buf);
        let name = u32::deserialize(buf);

        IrClassRef { parent, name }
    }
}

impl Serializable for IrField {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.flag.serialize(buf);
        self.name.serialize(buf);
        self.signature.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> IrField {
        let flag = u16::deserialize(buf);
        let name = u32::deserialize(buf);
        let descriptor = u32::deserialize(buf);

        IrField {
            flag,
            name,
            signature: descriptor,
        }
    }
}

impl Serializable for IrMethod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.signature.serialize(buf);

        self.flag.serialize(buf);

        self.locals.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> IrMethod {
        let name = u32::deserialize(buf);
        let descriptor = u32::deserialize(buf);

        let flag = u16::deserialize(buf);

        let locals = u16::deserialize(buf);

        IrMethod {
            flag,
            name,
            signature: descriptor,
            locals,
        }
    }
}

impl Serializable for IrMemberRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.parent.serialize(buf);
        self.name.serialize(buf);
        self.signature.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> Self {
        let parent = u32::deserialize(buf);
        let name = u32::deserialize(buf);
        let descriptor = u32::deserialize(buf);
        IrMemberRef {
            parent,
            name,
            signature: descriptor,
        }
    }
}

impl Serializable for IrMod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.entrypoint.serialize(buf)
    }

    fn deserialize(buf: &mut Deserializer) -> Self {
        let name = u32::deserialize(buf);
        let entrypoint = u32::deserialize(buf);
        IrMod { name, entrypoint }
    }
}

impl Serializable for IrModRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut Deserializer) -> Self {
        let name = u32::deserialize(buf);
        IrModRef { name }
    }
}

impl Serializable for IrBlob {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            IrBlob::Void => 0x00u8.serialize(buf),
            IrBlob::Bool => 0x01u8.serialize(buf),
            IrBlob::Char => 0x02u8.serialize(buf),
            IrBlob::U8 => 0x03u8.serialize(buf),
            IrBlob::I8 => 0x04u8.serialize(buf),
            IrBlob::U16 => 0x05u8.serialize(buf),
            IrBlob::I16 => 0x06u8.serialize(buf),
            IrBlob::U32 => 0x07u8.serialize(buf),
            IrBlob::I32 => 0x08u8.serialize(buf),
            IrBlob::U64 => 0x09u8.serialize(buf),
            IrBlob::I64 => 0x0Au8.serialize(buf),
            IrBlob::UNative => 0x0Bu8.serialize(buf),
            IrBlob::INative => 0x0Cu8.serialize(buf),
            IrBlob::F32 => 0x0Du8.serialize(buf),
            IrBlob::F64 => 0x0Eu8.serialize(buf),
            IrBlob::Obj(idx) => {
                0x0Fu8.serialize(buf);
                idx.serialize(buf);
            }
            IrBlob::Func(ps, ret) => {
                0x10u8.serialize(buf);
                ps.serialize(buf);
                ret.serialize(buf);
            }
            IrBlob::Array(content) => {
                0x11u8.serialize(buf);
                content.serialize(buf);
            }
        }
    }

    fn deserialize(buf: &mut Deserializer) -> Self {
        let code = u8::deserialize(buf);
        match code {
            0x00 => IrBlob::Void,
            0x01 => IrBlob::Bool,
            0x02 => IrBlob::Char,
            0x03 => IrBlob::U8,
            0x04 => IrBlob::I8,
            0x05 => IrBlob::U16,
            0x06 => IrBlob::I16,
            0x07 => IrBlob::U32,
            0x08 => IrBlob::I32,
            0x09 => IrBlob::U64,
            0x0A => IrBlob::I64,
            0x0B => IrBlob::UNative,
            0x0C => IrBlob::INative,
            0x0D => IrBlob::F32,
            0x0E => IrBlob::F64,
            0x0F => IrBlob::Obj(u32::deserialize(buf)),
            0x10 => {
                let ps = Vec::deserialize(buf);
                let ret = u32::deserialize(buf);
                IrBlob::Func(ps, ret)
            }
            0x11 => IrBlob::Array(u32::deserialize(buf)),
            _ => panic!("Cannot recognize blob with code {:0X}", code),
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
