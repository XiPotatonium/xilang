use std::io::Read;
use std::mem::transmute;

use super::blob::IrSig;
use super::code::CorILMethod;
use super::file::{IrFile, MAJOR_VERSION, MINOR_VERSION};
use super::member::{IrField, IrImplMap, IrMemberRef, IrMethodDef};
use super::module::{IrMod, IrModRef};
use super::param::IrParam;
use super::stand_alone_sig::IrStandAloneSig;
use super::ty::{IrTypeDef, IrTypeRef};

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
        self.param_tbl.serialize(&mut buf);
        self.stand_alone_sig_tbl.serialize(&mut buf);

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
        let param_tbl = Vec::deserialize(&mut buf);
        let stand_alone_sig_tbl = Vec::deserialize(&mut buf);

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
            param_tbl,
            stand_alone_sig_tbl,

            str_heap,
            usr_str_heap,
            blob_heap,

            codes,
        }
    }
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
impl_vec_serde!(IrParam);
impl_vec_serde!(IrStandAloneSig);
impl_vec_serde!(CorILMethod);
impl_vec_serde!(IrSig);
