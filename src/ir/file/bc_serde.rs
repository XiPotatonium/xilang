use std::io::Read;
use std::mem::transmute;
use std::slice::Iter;

use super::*;
use crate::blob::Blob;
use crate::inst::Inst;

impl IrFile {
    pub fn to_binary(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        self.major_version.serialize(&mut buf);
        self.minor_version.serialize(&mut buf);

        self.mod_tbl.serialize(&mut buf);
        self.modref_tbl.serialize(&mut buf);

        self.typedef_tbl.serialize(&mut buf);
        self.typeref_tbl.serialize(&mut buf);

        self.field_tbl.serialize(&mut buf);
        self.method_tbl.serialize(&mut buf);

        self.memberref_tbl.serialize(&mut buf);

        self.implmap_tbl.serialize(&mut buf);

        self.str_heap.serialize(&mut buf);
        self.usr_str_heap.serialize(&mut buf);
        self.blob_heap.serialize(&mut buf);

        self.codes.serialize(&mut buf);

        buf
    }

    pub fn from_binary(stream: Box<dyn Read>) -> IrFile {
        let mut buf = Deserializer::new(Box::new(stream.bytes().map(|r| r.unwrap())));

        let major_version = u16::deserialize(&mut buf);
        let minor_version = u16::deserialize(&mut buf);

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

        let implmap_tbl = Vec::deserialize(&mut buf);

        let str_heap = Vec::deserialize(&mut buf);
        let usr_str_heap = Vec::deserialize(&mut buf);
        let blob_heap = Vec::deserialize(&mut buf);

        let codes = Vec::deserialize(&mut buf);

        IrFile {
            minor_version,
            major_version,

            mod_tbl,
            modref_tbl,

            typedef_tbl: type_tbl,
            typeref_tbl,

            field_tbl,
            method_tbl,
            memberref_tbl,

            implmap_tbl,

            str_heap,
            usr_str_heap,
            blob_heap,

            codes,
        }
    }
}

pub trait IDeserializer {
    fn take_byte(&mut self) -> u8;
    fn take_bytes2(&mut self) -> [u8; 2];
    fn take_bytes4(&mut self) -> [u8; 4];
    fn take_bytes(&mut self, n: u32) -> Vec<u8>;
}

pub struct Deserializer {
    stream: Box<dyn Iterator<Item = u8>>,
    pub bytes_taken: u32,
}

impl Deserializer {
    pub fn new(stream: Box<dyn Iterator<Item = u8>>) -> Deserializer {
        Deserializer {
            stream,
            bytes_taken: 0,
        }
    }
}

impl IDeserializer for Deserializer {
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

pub trait ISerializable {
    fn serialize(&self, buf: &mut Vec<u8>);
    fn deserialize(buf: &mut dyn IDeserializer) -> Self;
}

impl ISerializable for u8 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(*self)
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> u8 {
        buf.take_byte()
    }
}

impl ISerializable for u16 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push((self >> 8) as u8);
        buf.push(*self as u8);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> u16 {
        let v = buf.take_bytes2();
        ((v[0] as u16) << 8) + (v[1] as u16)
    }
}

impl ISerializable for u32 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push((self >> 24) as u8);
        buf.push((self >> 16) as u8);
        buf.push((self >> 8) as u8);
        buf.push(*self as u8);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> u32 {
        let v = buf.take_bytes4();
        ((v[0] as u32) << 24) + ((v[1] as u32) << 16) + ((v[2] as u32) << 8) + (v[3] as u32)
    }
}

impl ISerializable for i8 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        unsafe { buf.push(transmute(*self)) }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> i8 {
        unsafe { transmute(buf.take_byte()) }
    }
}

impl ISerializable for i32 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let bytes = self.to_be_bytes();
        for b in bytes.iter() {
            buf.push(*b);
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> i32 {
        let bytes = buf.take_bytes4();
        i32::from_be_bytes(bytes)
    }
}

impl ISerializable for String {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u16).serialize(buf);
        for b in self.as_bytes() {
            b.serialize(buf);
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> String {
        let len = u16::deserialize(buf);
        let v = buf.take_bytes(len as u32);
        String::from_utf8(v).unwrap()
    }
}

impl ISerializable for Vec<u8> {
    fn serialize(&self, buf: &mut Vec<u8>) {
        (self.len() as u32).serialize(buf);
        for b in self.iter() {
            b.serialize(buf);
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Vec<u8> {
        let len = u32::deserialize(buf);
        buf.take_bytes(len)
    }
}

macro_rules! impl_vec_serde {
    ($t: ident) => {
        impl ISerializable for Vec<$t> {
            fn serialize(&self, buf: &mut Vec<u8>) {
                (self.len() as u32).serialize(buf);
                for v in self.iter() {
                    v.serialize(buf);
                }
            }

            fn deserialize(buf: &mut dyn IDeserializer) -> Self {
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
impl_vec_serde!(IrTypeDef);
impl_vec_serde!(IrTypeRef);
impl_vec_serde!(IrField);
impl_vec_serde!(IrMethodDef);
impl_vec_serde!(IrMemberRef);
impl_vec_serde!(IrImplMap);
impl_vec_serde!(CorILMethod);
impl_vec_serde!(Blob);

impl ISerializable for IrTypeDef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.flag.serialize(buf);

        self.fields.serialize(buf);
        self.methods.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> IrTypeDef {
        let name = u32::deserialize(buf);
        let flag = u32::deserialize(buf);
        let fields = u32::deserialize(buf);
        let methods = u32::deserialize(buf);
        IrTypeDef {
            name,
            flag,
            fields,
            methods,
        }
    }
}

impl ISerializable for IrTypeRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.parent.serialize(buf);
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let parent = u32::deserialize(buf);
        let name = u32::deserialize(buf);

        IrTypeRef { parent, name }
    }
}

impl ISerializable for IrField {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.flag.serialize(buf);
        self.name.serialize(buf);
        self.sig.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> IrField {
        let flag = u16::deserialize(buf);
        let name = u32::deserialize(buf);
        let descriptor = u32::deserialize(buf);

        IrField {
            flag,
            name,
            sig: descriptor,
        }
    }
}

impl ISerializable for IrMethodDef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.sig.serialize(buf);
        self.body.serialize(buf);

        self.flag.serialize(buf);
        self.impl_flag.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> IrMethodDef {
        let name = u32::deserialize(buf);
        let signature = u32::deserialize(buf);
        let body = u32::deserialize(buf);

        let flag = u16::deserialize(buf);
        let impl_flag = u16::deserialize(buf);

        IrMethodDef {
            name,
            body,
            sig: signature,
            flag,
            impl_flag,
        }
    }
}

impl ISerializable for IrMemberRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.parent.serialize(buf);
        self.name.serialize(buf);
        self.sig.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let parent = u32::deserialize(buf);
        let name = u32::deserialize(buf);
        let signature = u32::deserialize(buf);
        IrMemberRef {
            parent,
            name,
            sig: signature,
        }
    }
}

impl ISerializable for IrMod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
        self.entrypoint.serialize(buf)
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let name = u32::deserialize(buf);
        let entrypoint = u32::deserialize(buf);
        IrMod { name, entrypoint }
    }
}

impl ISerializable for IrModRef {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.name.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let name = u32::deserialize(buf);
        IrModRef { name }
    }
}

impl ISerializable for IrImplMap {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.member.serialize(buf);
        self.name.serialize(buf);
        self.scope.serialize(buf);
        self.flag.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let member = u32::deserialize(buf);
        let name = u32::deserialize(buf);
        let scope = u32::deserialize(buf);
        let flag = u16::deserialize(buf);
        IrImplMap {
            member,
            name,
            scope,
            flag,
        }
    }
}

impl ISerializable for CorILMethod {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.max_stack.serialize(buf);
        self.local.serialize(buf);
        self.insts.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let max_stack = u16::deserialize(buf);
        let local = u16::deserialize(buf);
        let insts = Vec::deserialize(buf);
        CorILMethod {
            max_stack,
            local,
            insts,
        }
    }
}

impl ISerializable for Blob {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            Blob::Void => 0x00u8.serialize(buf),
            Blob::Bool => 0x01u8.serialize(buf),
            Blob::Char => 0x02u8.serialize(buf),
            Blob::U8 => 0x03u8.serialize(buf),
            Blob::I8 => 0x04u8.serialize(buf),
            Blob::U16 => 0x05u8.serialize(buf),
            Blob::I16 => 0x06u8.serialize(buf),
            Blob::U32 => 0x07u8.serialize(buf),
            Blob::I32 => 0x08u8.serialize(buf),
            Blob::U64 => 0x09u8.serialize(buf),
            Blob::I64 => 0x0Au8.serialize(buf),
            Blob::UNative => 0x0Bu8.serialize(buf),
            Blob::INative => 0x0Cu8.serialize(buf),
            Blob::F32 => 0x0Du8.serialize(buf),
            Blob::F64 => 0x0Eu8.serialize(buf),
            Blob::Obj(idx) => {
                0x0Fu8.serialize(buf);
                idx.serialize(buf);
            }
            Blob::Func(ps, ret) => {
                0x10u8.serialize(buf);
                ps.serialize(buf);
                ret.serialize(buf);
            }
            Blob::Array(content) => {
                0x11u8.serialize(buf);
                content.serialize(buf);
            }
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let code = u8::deserialize(buf);
        match code {
            0x00 => Blob::Void,
            0x01 => Blob::Bool,
            0x02 => Blob::Char,
            0x03 => Blob::U8,
            0x04 => Blob::I8,
            0x05 => Blob::U16,
            0x06 => Blob::I16,
            0x07 => Blob::U32,
            0x08 => Blob::I32,
            0x09 => Blob::U64,
            0x0A => Blob::I64,
            0x0B => Blob::UNative,
            0x0C => Blob::INative,
            0x0D => Blob::F32,
            0x0E => Blob::F64,
            0x0F => Blob::Obj(u32::deserialize(buf)),
            0x10 => {
                let ps = Vec::deserialize(buf);
                let ret = u32::deserialize(buf);
                Blob::Func(ps, ret)
            }
            0x11 => Blob::Array(u32::deserialize(buf)),
            _ => panic!("Cannot recognize blob with code {:0X}", code),
        }
    }
}

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
            Inst::NewObj(idx) => {
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
            0x73 => Inst::NewObj(u32::deserialize(buf)),
            0x7B => Inst::LdFld(u32::deserialize(buf)),
            0x7D => Inst::StFld(u32::deserialize(buf)),
            0x7E => Inst::LdSFld(u32::deserialize(buf)),
            0x80 => Inst::StSFld(u32::deserialize(buf)),

            0xFE => {
                let inner_code = u8::deserialize(buf);
                match inner_code {
                    0x01 => Inst::CEq,
                    0x02 => Inst::CGt,
                    0x04 => Inst::CLt,
                    0x0C => Inst::LdLoc(u16::deserialize(buf)),
                    0x0E => Inst::StLoc(u16::deserialize(buf)),
                    _ => panic!("Unknown inst 0xFE{:X}", inner_code),
                }
            }
            _ => panic!("Unknown inst: 0x{:X}", code),
        }
    }
}

struct InstDeserializer<'i> {
    stream: Iter<'i, u8>,
    bytes_taken: u32,
}

impl<'i> InstDeserializer<'i> {
    fn new(insts: &Vec<u8>) -> InstDeserializer {
        InstDeserializer {
            stream: insts.iter(),
            bytes_taken: 0,
        }
    }
}

impl<'i> IDeserializer for InstDeserializer<'i> {
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
    pub fn new(max_stack: u16, local: u16, insts: Vec<Inst>) -> CorILMethod {
        let mut code = vec![];
        for inst in insts.iter() {
            inst.serialize(&mut code);
        }
        CorILMethod {
            max_stack,
            local,
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